"""core generic viewmanager functionality"""

import functools
import os
import re
from enum import Enum
from pathlib import Path
from types import SimpleNamespace
from typing import Any, Callable, NoReturn, Type
from uuid import uuid4

import django
from celery.utils.log import get_task_logger
from django.contrib import messages
from django.http import HttpResponse, HttpResponseRedirect, JsonResponse
from django.template.loader import get_template
from pydantic import BaseModel
from typing_extensions import ClassVar

from bemyerp.lib.dbsharing import DConnectionsProvider

import bemyerp.pssystem.constants as constants
import bemyerp.pssystem.messages as old_messages  # 🏷 105fra.119sel.001sel.073.django_messages.eval 🧟‍♂️
from bemyerp.lib.batch.common import BatchProxyConfig, get_batchconfig_by_taskname
from bemyerp.lib.batchhelper import decorate_task_url
from bemyerp.lib.common import Actual, EventCall, SubscriberEvent, SubscriberSlot, _MetaDirective, co
from bemyerp.lib.djangoutils import (
    check_accepts,
    get_li_rdbname,
    getJsonResponse404,
)
from bemyerp.lib.metaclass_viewmgr import MroErrorReport, ViewManagerMeta
from bemyerp.lib.template_generator_common import GeneratedResult
from bemyerp.lib.template_loader import loader_generator, mediator
from bemyerp.lib.utils import (  # noqa
    CompareType,  # noqa: F401
    DictPathInitializer,
    Dummy,
    HelperPS,
    NestedDictionary,
    PSAttrDict,
    Subber,
    _normalize_versionnames,
    cprint,
    debug_dump,
    fill_di_template,
    fill_template,
    is_subclass,
    nested_path_getter,
    ppp,
    rin,
    rindebug,
)
from bemyerp.lib.viewelements import MixinViewElements  # noqa: F401
from bemyerp.lib.viewmanager_providers import Provider
from bemyerp.pssystem.db import MultiDb, UserDb
from bemyerp.pssystem.dsl import Cardinality
from bemyerp.pssystem.exceptions import (
    AuthException,
    DataNotFoundException,
    InvalidConfigurationException,
    JsonReponseBearingException,
)
from bemyerp.pssystem.singletons import urlregistry
from bemyerp.pssystem.sql import PSQueryParts
from bemyerp.pssystem.tagreader import TagsReaderInstance
from bemyerp.pssystem.typing_ import T_mdb
from bemyerp.tests.helper_check_flagfile import check_flagfile
from bemyerp.websec.log_settings import Log

log = Log(__name__, prefix_lookup="views")


f_dump = None
if constants.IS_TEST:  # !!!TODO!!! get rid of later
    diroirawdump = os.getenv("diroirawdump")
    if diroirawdump:
        pa = Path(diroirawdump) / "debug.viewmanager"
        f_dump = functools.partial(debug_dump, dumpdir=pa)

DI_CLASSNAME_DEBUG = {}

DI_INST_PATH = {}


def logdead(msg, *args, **kwargs):  # custom deadgen
    log(f"{constants.LOG_PREFIX_DEBUG2.deadgen }{msg}", loglevel="info")


def cpdb(*args, **kwargs):
    """disabled conditional breakpoints"""


rpdb = breakpoints = cpdb

undefined = constants.undefined
PA_SCRIPT = Path(__file__)


class DataType(Enum):
    """fetching:  PS database or bme tables?"""

    PS = PSAttrDict
    postgres = SimpleNamespace


class DataProvider:
    """stub to indicate data provider"""

    cardinality: Cardinality
    datatype: DataType


class AcceptChecker:
    """indicates supported `HTTP_ACCEPT`
    checks request headers
    """

    accepted: Callable | str | set[str]

    _accept_check = undefined

    @property
    def accept_check(self) -> Callable:
        if self._accept_check is undefined:
            accepted = self.accepted

            if isinstance(accepted, str):
                s_provided = set(accepted.split(","))

                def check(request):
                    HTTP_ACCEPT = request.META.get("HTTP_ACCEPT", "")
                    if not HTTP_ACCEPT:
                        return

                    s_accepted = set(HTTP_ACCEPT.split(";")[0].split(","))

                    if s_accepted and not s_provided & s_accepted:
                        data = dict(
                            message=f"{self.f_view.__name__} @ {request.path} supports:{s_provided} : client.accept{s_accepted}",
                            status=406,
                        )
                        # TODO really should return a Response matching Accept...
                        raise JsonReponseBearingException(ValueError(data), status=406)

                self._accept_check = check
            elif callable(accepted):
                self._accept_check = accepted
            else:
                raise NotImplementedError()

        return self._accept_check

    def prep(self, *args, **kwargs):
        super().prep(*args, **kwargs)
        self.accept_check(self.request)


class Phase(str, Enum):
    """indicates when variables are available
    for example, data computed on fetched results
    are not available during `__init__`
    🧟‍♂️? Maybe not so necessary now that only `finalize`
    sets `di_settings` and `di_context`
    """

    init = "__init__"
    final = "final"


if constants.IS_TEST:
    DI_CLASSNAME_DEBUG = {}
    DI_INST_REQUEST_PATH = {}


class VueManager2:
    def __repr__(self) -> str:
        return f"{self.__class__.__name__}"

    _meta_todos_merge_tuples = _MetaDirective(attrname="cls_subscribers", typeswanted=(SubscriberSlot, str), merge_func="merge_slots")

    initializer = DictPathInitializer()

    def vmc_dict_path(self, path: str):  # , value: Any = undefined):
        """vmgr core services"""

        assert path in co.datapaths

        return self.initializer.ini_dict_path(self, path, breakpoints=breakpoints)

    _connections_ = None

    @property
    def _connections(self):
        """Property to access the DConnectionsProvider instance."""
        return self._connections_

    @_connections.setter
    def _connections(self, connections):
        """watchdog to catch the transition"""
        try:
            if self._connections_ is not None and connections is not self._connections_:
                raise ValueError(
                    f"{self}.dont reset {self._connections_=}[{id(self._connections_)}] with {connections=}[{id(connections)}]"
                )

            self._connections_ = connections
        # pragma: no cover pylint: disable=unused-variable
        except (Exception,) as e:
            if cpdb():
                breakpoint()
            raise

    connections = DConnectionsProvider()

    # prop

    cprint = cprint

    classname = "?"  #!!!TODO!!! debug only 🧟‍♂️

    phase: Phase = Phase.init

    _fetched_already = undefined

    cls_di_context = {
        "use_v2": None,
        "classname": None,
        "realclassname": None,
    }

    @classmethod
    def _get_debug_function(cls):
        funcname = "_debug_flog"
        flog = getattr(VueManager2, funcname, undefined)
        if flog is undefined:
            from bemyerp.lib.utils import import_string

            # howto- activate special custom viewmanager logging
            vmgrlogger = os.getenv("vmgrlogger")
            if vmgrlogger:
                flog = import_string(vmgrlogger)
            else:
                flog = None
            setattr(VueManager2, funcname, flog)
            return flog
        else:
            return flog

    @classmethod
    def debug_data(
        cls,
        classname: str,
        *args,
        llog: Callable | None = None,
        msg="",
        prefix="",
        data=undefined,
        fnprefix="",
        use_sequence=True,
        **kwargs,
    ):
        try:
            if (not msg or data is undefined) and constants.IS_TEST:
                breakpoint()
            log2 = llog or log

            flog = cls._get_debug_function()
            if flog:
                out = flog(
                    vmgr=classname,
                    prefix=prefix,
                    msg=msg,
                    llog=llog,
                    fnprefix=fnprefix,
                    data=data,
                    use_sequence=use_sequence,
                )
                log2(out)
            else:
                log2(f"{prefix}{msg}")
        except (Exception,) as e:
            log2(e)
            pass

    def debug(
        self,
        *args,
        llog: Callable | None = None,
        msg="",
        prefix="",
        data=None,
        fnprefix="",
        use_sequence=True,
        **kwargs,
    ):
        "do-nothing, replace externally"
        try:
            if not msg and constants.IS_TEST:
                breakpoint()
            log2 = llog or log

            flog = self._get_debug_function()
            if flog:
                out = flog(
                    vmgr=self,
                    prefix=prefix,
                    msg=msg,
                    llog=llog,
                    fnprefix=fnprefix,
                    data=data,
                    use_sequence=use_sequence,
                )
                log2(out)
            else:
                log2(f"{prefix}{msg}")
        except (Exception,) as e:
            if constants.IS_TEST:
                log(f"logging exception {e=}", loglevel="exception")
                breakpoint()
                pass
            pass

    def use_v2(self):
        return check_flagfile()

    def realclassname(self):
        return self.__class__.__name__

    def classname(self):
        # log(f"{self}.classname:deprecated.  use `_normalize_classname`", loglevel="warning")
        return self._normalize_classname()

    def _normalize_classname(self):
        return _normalize_versionnames(self)

    def _debug_register_instance(self):
        if constants.IS_TEST:
            DI_CLASSNAME_DEBUG[self._normalize_classname()] = self

    def __init__(self, *args, **kwargs):
        "do-nothing. the action is in subclasses"
        self._config_basetype: dict[Type, Any] = {}

        attrname = "_config_basetype"

        cls_ = type(self)
        _config_basetype = getattr(cls_, attrname, None)
        if _config_basetype is None:
            cls_._config_basetype: dict[Type, Any] = {}

        self.event_subscribers: dict[SubscriberEvent, tuple[EventCall]] = SubscriberSlot.get_subscribers(self)

        if constants.IS_TEST:
            try:
                path = kwargs.get("request").path

                assign = True
                if "compare" in path and self.endpointtype == self.endpointtype.detail:
                    assign = False

                # avoid assigning sub-viewmanagers like PSOPRDEFN_Detail => Usergroup_Detail...
                if DI_INST_REQUEST_PATH.get(path):
                    assign = False

                if assign:
                    DI_INST_REQUEST_PATH[path] = self
            except (AttributeError,):
                pass

    def set_user_msg_immediate(self, html: str, level="WARNING", extra_tags=""):  # 🏷 105fra.119sel.001sel.073.django_messages.eval
        """used to set user message prior to a redirect"""
        try:
            level = level.upper()
            level = getattr(messages, level)

            extra_tags += f"msg{level}"

            messages.add_message(request=self.request, level=level, message=html, extra_tags=extra_tags)

        # pragma: no cover pylint: disable=unused-variable
        except (Exception,) as e:
            if cpdb():
                breakpoint()
            raise

    def prep(self, di_spike={}, *args, **kwargs):
        fn = getattr(super(), "prep", None)
        if callable(fn):
            fn(*args, **kwargs)

        return self.di_context

    def cleanup(self):
        connections = self.connections
        connections.cleanup()

    def dowork(self, *args, **kwargs):
        """do-nothing hook"""
        funcname = "dowork"
        f_super = getattr(super(), funcname, None)
        if f_super:  # pragma: no cover - this is an error condition
            msg = MroErrorReport.mro_report_error(self.__class__, basecls=VueManager2)
            raise InvalidConfigurationException(
                f"{self}[VueManager2]:fetch - there should be no `super().{funcname}` but there is: {f_super}.\n{msg}"
            )

    def fetch(self, *args, **kwargs):
        """do-nothing hook"""

        from_prep = kwargs.get("from_prep")

        f_super = getattr(super(), "fetch", None)
        if f_super:  # pragma: no cover - this is an error condition
            msg = MroErrorReport.mro_report_error(self.__class__, basecls=VueManager2)
            raise InvalidConfigurationException(
                f"{self}[VueManager2]:fetch - there should be no `super().fetch` but there is: {f_super}.\n{msg}"
            )
        if not from_prep:
            self.call_subscribers(SubscriberEvent.post_fetch)

    def fetch2(self, *args, **kwargs):
        """do-nothing hook"""

    def do_integrity_checks(self):
        """do-nothing hook"""

    def finalize(self, *args, **kwargs) -> None:
        """finalize"""

        # 🏷 105.framework.031.use_pubsub_forcalls.enhancement - call entry_finalizers

        self._prep_dict(self.cls_di_context, self.di_context)
        self._prep_dict(self.cls_di_settings, self.di_settings)

        fn = getattr(super(), "finalize", None)
        if callable(fn):
            fn(*args, **kwargs)

        self.di_context["j_settings"] = self.di_settings

        # 🏷 105.framework.031.use_pubsub_forcalls.enhancement - call exit_finalizers

    def post_finalize(self, *args, **kwargs):
        """stuff REALLY coming last"""

        fn = getattr(super(), "post_finalize", None)
        if callable(fn):
            fn(*args, **kwargs)

    def _prep_dict(self, cls_dict, inst_dict, subber=None):
        subber = subber or self.subber
        prefix = f"{self}[VueManager2]._prep_dict:"

        classname = self._normalize_classname()

        for name, val in cls_dict.items():
            oi = False
            if breakpoints("_prep_dict", {"classattrib": (classattrib := f"{classname}.{name}")}):  # pragma: no cover
                # print(f"\n\n🔬classattrib:{classattrib}")
                oi = True
                breakpoint()
                pass
            if callable(val):
                value = val(self)
            elif isinstance(provider := getattr(self, name, None), Provider):
                msg = f"{provider=}@{self}.{name}"
                value = provider.provide(self, cls_dict, inst_dict, subber, name, val)
                if provider.continue_after:
                    continue
            elif isinstance(val, dict) and (
                # this resolves to `_provide_urls` if `name` is `urls`
                f_calc := getattr(self, f"_provide_{name}", None)
            ):
                # yay, someone else takes responsibility...
                f_calc(cls_dict, inst_dict, subber, name, val)
                # so dont forget to bail out of this attribute's evaluation
                continue
            elif isinstance(val, Actual):
                from django.utils.safestring import mark_safe

                value = mark_safe(str(val))
                pass
            elif isinstance(val, str):
                value = subber[val]
            elif isinstance(val, (int, dict)):
                value = val
            elif val is None:
                value = subber[name]
            elif val is undefined:
                # we don't want to set this variable after all
                continue
            else:
                raise InvalidConfigurationException(f"{prefix} {name=} {val=}[{type(val)}] can't be prepped @ {inst_dict=}")
            inst_dict[name] = value

    def call_subscribers(self, event: SubscriberEvent, **kwargs):
        """call the applicable subscriber function"""
        try:
            prefix = f"{self}.call_subscribers:"
            _inst_previously_called_subscribers: set[EventCall] = getattr(self, "_inst_previously_called_subscribers", undefined)
            if _inst_previously_called_subscribers is undefined:
                self._inst_previously_called_subscribers = _inst_previously_called_subscribers = set()

            event_subscribers = getattr(self, "event_subscribers", {})

            # multiple subscribers can listen to the same event
            subscribers: list[EventCall] = event_subscribers.get(event, [])
            if not subscribers:
                return

            funcname = event.funcname

            mdb = getattr(self, "mdb", None)
            for subscriber in subscribers:
                # enforce once-only calling, most to support `inst = viewmanager.fetch()` in the views
                if subscriber in _inst_previously_called_subscribers:
                    continue

                # this will error out if the function doesnt exist.  don't subscribe if you are not providing the service
                f_notify = getattr(subscriber, funcname)
                f_notify(vmgr=self, mdb=mdb, event=event, **kwargs)
                _inst_previously_called_subscribers.add(subscriber)

        except (Exception,) as e:
            if cpdb():
                breakpoint()
                pass
            raise

    def set_payload(
        self,
        di_view: dict[str, Any],
        di_context_seed: dict[str, Any] = {},
        di_settings: dict[str, Any] = {},
    ):
        """
        makes data available to the viewmanager
        - di_view is available to the viewmanager
        - di_context is passed to template.render(self.di_context, self.request)
        - di_context_seed ends up in the page's `json settings`, for use by JS/Vue
        """
        self.di_context = di_context_seed.copy()
        self.di_settings.update(**di_settings)
        self.di_view.update(**di_view)
        return self

    def prep_all(self, persist=True, f_callback_pregen=None):
        """does everything except handling the template"""

        classname = self._normalize_classname()

        prefix = f"{classname}.prep_all[VueManager2]:"

        self.debug(f"{prefix}", msg="ante.prep", fnprefix="m01")
        self.prep()
        self.debug(f"{prefix}", msg="post.prep", fnprefix="m02")

        connections = self.connections

        if self._fetched_already is undefined:
            if breakpoints("VueManager2_fetch", {"classname": classname}):  # pragma: no cover
                # print(f"\n\n🔬🔬🔬classname:{classname}")
                breakpoint()
                pass

            self.fetch(f_view=self.f_view, from_prep=True, connections=connections)
            self.debug(f"{prefix}", msg="post.fetch", fnprefix="m03")

        connections = self.connections

        self.call_subscribers(SubscriberEvent.post_fetch)

        self.dowork()
        self.debug(f"{prefix}", msg="post.dowork", fnprefix="m04")
        self.do_integrity_checks()
        self.phase = Phase.final
        self.debug(f"{prefix}", msg="ante.finalize", fnprefix="m05")

        self.call_subscribers(SubscriberEvent.ante_finalize)
        self.finalize()
        self.debug(f"{prefix}", msg="post.finalize", fnprefix="m06")

        connections = self.connections

        self.post_finalize()
        self.call_subscribers(SubscriberEvent.post_finalize)

        connections = self.connections

        self.debug(f"{prefix}", msg="post.post_finalize", fnprefix="m07")

        if breakpoints("VueManager2_return", {"classname": classname}):  # pragma: no cover
            # print(f"\n\n🔬🔬🔬classname:{classname}")
            breakpoint()
            pass

        self.cleanup()

    def HttpResponse(self, main_template=None, persist=True, f_callback_pregen=None, status_code=None):
        html = self.prep_html(main_template, persist, f_callback_pregen)
        self._debug_register_instance()
        return HttpResponse(html, status=status_code)

    def prep_html(self, main_template: str | None = None, persist=True, f_callback_pregen=None):
        try:
            t = None
            main_template = main_template or self.template_name
            self.prep_all(persist, f_callback_pregen)
            self.template = t = django.template.loader.get_template(main_template)
            if f_dump:
                # howto- deep conditional debug dump to yaml with fallbacks to pprint
                # via `utils.debug_dump`
                skip_by_regex = {
                    re.compile("^inst"): None,
                    re.compile("^li_"): None,
                    re.compile("^mdb"): None,
                }
                subdirname = self.f_view.__name__
                attributes = "viewelements cssids templates".split()
                f_dump(
                    subdirname=subdirname,
                    src=self.di_context,
                    attributes=[None, "viewelements"],
                    prefix="context",
                    dict_filter=skip_by_regex,
                )
                f_dump(
                    subdirname=subdirname,
                    src=self.di_view,
                    attributes=[None, "viewelements"],
                    prefix="view",
                    richdump=True,
                    dict_filter=skip_by_regex,
                    typedump="yaml",
                )
                f_dump(
                    subdirname=subdirname,
                    src=self.di_settings,
                    attributes=[None, "viewelements"],
                    prefix="settings",
                )

            data = self.di_context.get("viewelements", {}).get("tabs", {})

            self.debug(f"{self}[VueManager2].prep_html", msg="ante.render0", fnprefix="m80", data=data)
            self.debug(f"{self}[VueManager2].prep_html", msg="ante.render", fnprefix="m90")
            html = t.render(self.di_context, self.request)
            self.debug(f"{self}[VueManager2].prep_html", msg="post.render", fnprefix="m95")
            return html
        # pragma: no cover pylint: disable=unused-variable
        except (DataNotFoundException,) as e:
            raise
        except (Exception,) as e:
            if cpdb():
                breakpoint()
            raise

    def _set_annotations(self, subber: Subber, **kwargs):
        firstkeys = kwargs.keys() & self.annos.keys()

        for attrname in firstkeys:
            setattr(self, attrname, kwargs[attrname])

        # precedence - what was passed as kwargs
        # , what the view's decorations contains and the class itself

        for k, anno in self.annos.items():
            if k in firstkeys:
                continue
            try:
                v = subber[k]
                if can_isinstance(anno):
                    if is_subclass(anno, BaseModel) and isinstance(v, dict):
                        v = anno(**v)

                    if not isinstance(v, anno):
                        # pragma: no cover
                        raise InvalidConfigurationException(f"{self}[{self.__class__.__name__}].`{k}`. expected type {anno}.  got {v}")

            # pragma: no cover pylint: disable=unused-variable
            except (KeyError,) as e:
                required_by = ""
                for cls in self.__class__.mro():
                    annotations = getattr(cls, "__annotations__", {})
                    if k in annotations:
                        required_by = f", required by {cls}"

                raise InvalidConfigurationException(
                    f"{self}[{self.__class__.__name__}].missing variable `{k}`{required_by}.  Should be on kwargs, f_view or on class"
                )

            if isinstance(v, (set, list, dict)):
                v = v.copy()
            try:
                setattr(self, k, v)
            # pragma: no cover pylint: disable=unused-variable
            except (AttributeError,) as e:
                vc = getattr(self.__class__, k, undefined)
                if isinstance(vc, property):
                    pass
                else:
                    raise


@functools.lru_cache
def can_isinstance(anno) -> bool:
    """Can an annotation provide `isinstance` checking?  Anything with `Any` can't"""
    try:
        isinstance(None, anno)
        return True
    except (TypeError,) as e:
        return False


class MinimalManager(VueManager2, metaclass=ViewManagerMeta):
    "use for json fetches and lookups"

    def __init__(self, request, f_view, **kwargs):
        self.f_view = f_view
        self.request = request
        self.subber = Subber(self.f_view, kwargs, self)
        subber = Subber(kwargs, f_view, self)
        self._set_annotations(subber, **kwargs)


class BaseManager(VueManager2, metaclass=ViewManagerMeta):
    # the template for local navigation
    templatename_localnav: str

    # required when mixing in `class.GenTemplateMix`
    generator_config_scalar = constants.FNP_CONFIG_GENERATOR_ROWCOL_YAML
    generator_config_table = constants.FNP_CONFIG_GENERATOR_TABLE_YAML

    bundle_name: str

    skin: str = ""

    def _username(self):
        return self.request.user.username

    def _is_superuser(self):
        return self.request.user.is_superuser

    cls_di_context = dict(
        title=None,
        current_app_label=None,
        templatename_localnav=None,
        username=_username,
        bundle_name=None,
        is_superuser=_is_superuser,
    )

    def finalize(self) -> None:
        super().finalize()
        self.di_context["viewdebug"] = self.request.GET.get("debug")
        # if tabulator_settings:

    def _li_rdbname(self):
        """get the rdbnames available to the user"""
        res = self.request.session.get("li_rdbname")
        # log(f"{self}._li_rdbname:785.from_session=>{res}", loglevel="warning")
        if not res:
            res = li_rdbname = get_li_rdbname(self.request) or []
            self.request.session["li_rdbname"] = li_rdbname
        # log(f"{self}._li_rdbname:=>{res}", loglevel="warning")
        return res

    cls_di_settings = dict(
        is_superuser=_is_superuser,
        li_rdbname=_li_rdbname,
    )

    def __repr__(self) -> str:
        return f"{self.__class__.__name__}"

    def __init__(self, request, f_view, **kwargs):
        self.f_view = f_view
        self.di_view = {}
        self.request = request
        self.subber = Subber(self.di_view, self.f_view, kwargs, self)
        subber = Subber(kwargs, f_view, self)
        self._set_annotations(subber, **kwargs)
        self.di_settings: dict[str, Any] = dict(li_user_message=[])
        self.di_context = {}

        self.di_event_subs = {}

        fn_super = getattr(super(), "__init__", None)
        if callable(fn_super):
            fn_super(request=request, f_view=f_view, **kwargs)

    def post_finalize(self, *args, **kwargs):
        """stuff REALLY coming last"""
        try:
            prefix = f"{self}[BaseManager].post_finalize"

            fn = getattr(super(), "post_finalize", None)
            if callable(fn):
                fn(*args, **kwargs)

        # pragma: no cover pylint: disable=unused-variable
        except (Exception,) as e:
            if cpdb():
                breakpoint()
            raise

    def set_user_msg(self, heading, body="", panel_class="warning"):  # 🏷 105fra.119sel.001sel.073.django_messages.eval
        """used to set user message prior to a redirect"""
        try:
            self.set_user_msg_immediate(html=heading, level=panel_class)
        # pragma: no cover pylint: disable=unused-variable
        except (Exception,) as e:
            if cpdb():
                breakpoint()
            raise

    def redirect_exception(self, exc, urlname, subber, heading, **kwargs):
        t_url = urlregistry.get(urlname)
        if t_url:
            url = fill_template(t_url, subber)
        else:
            url = django.urls.reverse(urlname, kwargs=dict(rdbname=subber.get("rdbname")))
        self.set_user_msg(heading)
        response = HttpResponseRedirect(url)
        exc.response = response
        return exc


class RDBMix(BaseManager):
    """The primary  purpose of this Mixin class is to drive `super().__init__` calls.
    It is intended for single-database pages - i.e. Detail, Compare, List.
    Not XDBs - which will get a different Mixin.

    Shapes the incoming arguments - by adding rdbname to it and queries db status
    Do not put much else on it, because it will bypass other Mixin's methods by routing them
    up to BaseManager.

    """

    helperps = HelperPS()

    cls_di_context = dict(li_missing_record=None, rdbname=None)
    cls_di_settings = dict(
        rdbname=None,
    )

    mdb: MultiDb
    rdbname: str

    def do_integrity_checks(self):
        """check for error conditions"""

        if self.rdbname:
            self.check_missing_batches(self.rdbname)

    def check_missing_batches(  # 🏷 105fra.119sel.001sel.073.django_messages.eval
        self,
        rdbname,
        batch_needs: set[str] = None,
        T_MSG: set = old_messages.T_MISSING_BATCH_REQ,
        add_to_messages=True,
    ):
        """
        check for missing batches and inject them into the user messages
        """

        # if not given, grab it from the f_view
        if batch_needs is None:
            # defaults to None if not found which then gets changed into an empty set
            batch_needs = getattr(getattr(self, "f_view", None), "batch_needs", None) or set()

        # else:
        batch_needs = set(batch_needs)
        status = getattr(self, "rdbstatus", None) or getattr(self, "sysstatus", None)
        bad_batches = getattr(status, "bad_batches", None)
        if not bad_batches:
            return set()
        missings = batch_needs & bad_batches
        for batchname in missings:
            taskname = constants.DI_ALIAS_BATCH_TASKNAME.get(batchname)

            heading = f"""<span class="mr-2">Missing {batchname}</span>"""
            if taskname:
                batchproxy = get_batchconfig_by_taskname(taskname)  # 🏷 105fra.119sel.001sel.073.django_messages.eval

                css = dict(run_htmx="inline-block")

                batch = batchproxy.get_actual_proxy(
                    context=dict(rdbname=rdbname, css=css),
                )

                template = get_template("pssystem/batch_run_and_status.html")
                html = template.render(context=dict(batch=batch, css=css), request=self.request)

                self.set_user_msg_immediate(html=f"{heading}{html}", level="warning")
            else:
                self.set_user_msg_immediate(html=heading, level="warning")

        return missings

    _rdbstatus = undefined

    @property
    def rdbstatus(self):
        if self._rdbstatus is undefined:
            self._rdbstatus = constants.rdbstatus.get(rdbname=self.rdbname)
        return self._rdbstatus

    def __init__(self, request: Any, mdb: T_mdb, f_view, **kwargs):
        super().__init__(request=request, f_view=f_view, mdb=mdb, rdbname=mdb.rdbname, **kwargs)
        self.li_missing_record = []


class FormSearchMix(DataProvider):
    form_name: str = "bmeFormSearch"
    rootname: str
    form: Any

    cardinality = Cardinality.vector

    __fetched_already = False

    _select_clause = undefined  # type: ignore

    def prep(self, *arg, **kwargs):
        f_super = getattr(super(), "prep", None)
        if f_super:
            f_super(*arg, **kwargs)
        # allows for cases where the form isn't actually being used
        if self.form:
            self.form.update_settings(self.di_settings, self.form_name)
        return self.di_settings

    @property
    def select_clause(self) -> list[str]:
        if self._select_clause is undefined:
            self.__class__._select_clause = None  # type: ignore
            di_gen_templates = getattr(self.f_view, "gen_templates", {})
            di_json_tabulators = getattr(self.f_view, "json_tabulators", {})
            # !!!TODO!!! this implicit naming stuff is NOT great.  consider specifying it
            # explicitly
            directive_template = (
                di_gen_templates.get("list_template") or di_json_tabulators.get("list_template") or di_json_tabulators.get("list")
            )
            if not directive_template:
                return self._select_clause  # type: ignore

            raise NotImplementedError("see prototype saved files")
        return self._select_clause  # type: ignore


class GuardMix(DataProvider, MinimalManager, AcceptChecker):
    """use this to control access and existence on views
    that don't want to instantiate a full Manager
    """

    # guard_content_type: str
    accepted = "application/json"
    objecttype: int

    mdb: MultiDb
    rdbname: str

    qp_cls: type

    cardinality = Cardinality.scalar

    tmpl_404_msg: str = "%(recname)s Not found: %(di_keys)s"

    def __init__(self, request: Any, mdb: T_mdb, f_view, **kwargs):
        super().__init__(request=request, f_view=f_view, mdb=mdb, rdbname=mdb.rdbname, **kwargs)

    def _rescue_missing_key(self, context, key, subber: Subber) -> NoReturn:
        """
        allows the specific subclass to adjust lookup behavior
        return True if the query and/or criteria can be adjusted
        any other result, including throwing an exception will be treated as InvalidConfigurationException
        """

        raise NotImplementedError("this needs to be subclassed")

    def _enhance_diagnostics(self, e, context, subber):
        pass

    def _calc_redirect_url(self, exc, context, subber):
        if exc.__class__ in (DataNotFoundException,):
            app_url = {"pssecurity": "security"}.get(self.appname)
            urlname = f"{app_url}:{self.redirect_pattern_name}"
            template = urlregistry.get(urlname)
            if not template:  # pragma: no cover
                raise InvalidConfigurationException(f"{urlname=} known: {urlregistry.registry.keys()}")
            url = fill_template(template, subber)
            message = fill_template(self.tmpl_404_msg, subber)
        elif exc.__class__ in (AuthException,):
            message = "you either do not not have access to Database.%(rdbname)s.  Or it doesn't exist.  The databases you can access" % (
                subber
            )
            url = "/databases"
        else:
            message = "unhandled exception:%s" % (exc)
            url = "/"
        f_set_usermessage = getattr(self, "set_user_msg", None)
        if f_set_usermessage:
            f_set_usermessage(heading=message)
        else:
            self.set_user_msg(context.request, heading=message)
        return url

    def _decorate_exception(self, exc: Exception, context, subber) -> None:
        """# !!!TODO!!!" extend this to other configuration/data not found or batch missing
        but this needs to be in the bemyerp.pssystem.exceptions module.
        """

        if hasattr(exc, "error_response"):
            return

        self._enhance_diagnostics(exc, context, subber)

        guard_content_type = self.__dict__.get("guard_content_type", undefined)

        if guard_content_type == "html" or (not context.json_only and check_accepts(context.request, "html")):
            url = self._calc_redirect_url(exc, context, subber)
            exc.error_response = HttpResponseRedirect(url, status=302)
            raise

        elif guard_content_type == "json" or context.json_only or check_accepts(context.request, "json"):
            # need a better status code?  or not?

            if isinstance(exc, AuthException):
                exc.error_response = getJsonResponse404(fill_template("details: %s", exc.row_auth))
            elif isinstance(exc, DataNotFoundException):
                exc.error_response = getJsonResponse404(
                    fill_template(
                        self.tmpl_404_msg,
                        subber,
                        dict(recname=constants.di_objecttype_recname[self.objecttype]),
                    )
                )

            raise
        # pragma: no cover
        raise InvalidConfigurationException("could not determine appropriate 404 response content_type")

    def _inst_fetch(self, context, db, qp, di_keys, subber):
        try:
            return qp.fetch(db, wrap=True, **di_keys)

        except AuthException as e:
            self._decorate_exception(e, context, subber)
            raise
        except DataNotFoundException as e:  # pragma: nocover
            self._decorate_exception(e, context, subber)
            raise

    def _check_auth(self, db, di_crit):
        "check if user is authorized for rdb"
        row_auth = db.selectone("select * from check_auth_rdb(%(rdbname)s, %(username)s)", di_crit)

        if row_auth.status_code >= 400:
            exc = AuthException("details:%s" % dict(row_auth))
            exc.row_auth = row_auth
            raise exc

    def fetch(self, request, check_db_access=True, json_only=False, **kwargs):
        """get an Usergroup instance"""
        try:
            qp_cls = keys = context = None

            context = Dummy(check_db_access=check_db_access, json_only=json_only, **kwargs)

            qp_cls = self.qp_cls
            keys = qp_cls.keys
            context.qp = qp = qp_cls()

            #######################################################
            # some generic parameter adjustments
            #######################################################

            context.mdb = mdb = kwargs.get("mdb")

            context.request = request  # = kwargs.get("request")

            # wkseq should nearly always be 0
            if not isinstance(qp, PSQueryParts):
                if "wkseq" not in kwargs:
                    kwargs["wkseq"] = 0

            context.rdbname = rdbname = kwargs.get("rdbname")
            if not rdbname and mdb:
                # might even want to always reassign so that
                # you don't come in one way and look elsewhere...
                context.rdbname = kwargs["rdbname"] = mdb.rdbname

            context.sysdb = sysdb = kwargs.get("sysdb") or getattr(mdb, "sysdb", None) or getattr(mdb, "djangodb", None)

            context.di_keys = di_keys = {}

            subber = Subber(di_keys, context)

            for key in keys:
                try:
                    di_keys[key] = kwargs[key]
                except (KeyError,) as e:
                    try:
                        saved = self._rescue_missing_key(context, key, subber)
                    except InvalidConfigurationException as e:  # pragma: no cover
                        raise
                    except (DataNotFoundException,) as e:  # pragma: nocover
                        self._decorate_exception(e, context, subber)
                        raise
                    except (Exception,) as e:  # pragma: no cover pylint: disable=unused-variable, broad-except
                        saved = False

                    if not saved:  # pragma: no cover
                        raise InvalidConfigurationException(
                            "%s.get_inst():missing key:%s.  querypart:%s expects: %s given:%s" % (self, key, qp, qp.keys, kwargs)
                        )

            if check_db_access and not mdb and "rdbname" in keys:
                # if you have the mdb already, then rdb access has been validated there...

                username = getattr(request, "username", None) or request.user.username
                di_sub = dict(rdbname=rdbname, username=request.username)

                with UserDb.get() as db:
                    row_auth = self._check_auth(db, di_sub)

            if isinstance(qp, PSQueryParts):
                if not mdb:
                    raise NotImplementedError("need to implement mdb acquisition")
                inst = self._inst_fetch(context, mdb, qp, di_keys, subber)
                return inst
            else:
                inst = self._inst_fetch(context, sysdb, qp, di_keys, subber)
                return inst

        except (DataNotFoundException, AuthException) as e:
            raise
        except InvalidConfigurationException as e:
            di_debug = dict(qp_cls=qp_cls, keys=keys, kwargs=kwargs)
            # ppp(di_debug, "\n\n\ndebug")
            raise


#################################################################
# Batch runner via pydantic
#################################################################


class DirectRunner:
    """runs a batch inline"""

    rootname = "batchresults"
    params: Any = None

    options_cls: type[BaseModel]
    fn_run: Callable[[BaseModel], Any]

    def post_finalize(self, *args, **kwargs):
        fn = getattr(super(), "post_finalize", None)

        if breakpoints("post_finalize", {"always": 1}):  # pragma: no cover
            # print(f"\n\n🔬🔬🔬always:{1}")
            breakpoint()
            pass

        if fn:
            fn(*args, **kwargs)
        try:
            vmgr = nested_path_getter(self.di_settings, "batchform.data.vmgr")
            if vmgr:
                self.di_settings["batchform"]["data"].pop("vmgr")
        # pragma: no cover pylint: disable=unused-variable
        except (AttributeError, KeyError) as e:
            pass

    def fetch(self, params: dict[str, Any] | BaseModel | None = None, *args, **kwargs):
        """runs the task"""
        res = getattr(self, self.rootname, undefined)
        if res is not undefined:
            return res
        params = params or self.params
        if isinstance(params, dict):
            self.options = self.options_cls(vmgr=self, **params)
        elif params is None:
            tmp = self.options_from_none()
            self.options = self.options_cls(vmgr=self, **tmp)
        else:
            self.options = params
        res = self.fn_run(self.options)
        setattr(self, self.rootname, res)
        self.fetch2(*args, **kwargs)
        return res

    def options_from_none(self):
        return dict(rdbname=self.rdbname)


class CeleryBaseRunner:
    options_cls: type[BaseModel]
    task: Callable
    force_sync: bool = False

    def run(self, data: dict[str, Any] | BaseModel | None = None, *args, **kwargs):
        """runs the task"""
        prefix = f"{self}.run:"
        # since
        data = data or self.data
        if data is None:
            return

        fn = self.task.apply if self.force_sync else self.task.apply_async

        # log(f"{prefix}1345 {self.force_sync=} apply_async. {fn=}", loglevel="warning")

        ########
        # If using apply_async, we can get the AsyncResult and its id before execution
        # But to pass the task_id to the function, we need to use Celery's link or pass it as an argument
        # Here, we inject the task_id into the options if possible

        di = {}
        if not self.force_sync:
            # Generate a task id and pass it as an argument
            task_id = str(uuid4())

            if isinstance(data, dict):
                data["task_id"] = task_id
            else:
                try:
                    data.task_id = task_id
                except (AttributeError,) as e:
                    log(f"{prefix} : exception:{e}")

            di["task_id"] = task_id

            # log(f"{prefix} {di=} apply_async", loglevel="warning")

        if not isinstance(data, BaseModel):
            self.options = self.options_cls(**data)
        else:
            self.options = data

        if getattr(self, "pass_pydantic_instance", False):
            self.tmp = res = fn([self.options], **di)
        else:
            di_dump = self.options.model_dump()

            self.tmp = res = fn([di_dump], **di)

        return res

    DI_STATE_HTTP_STATUS = {"SUCCESS": 200, "PENDING": 202, "FAILURE": 500}

    tmpl_msg = "Task.%(taskname)s status:%(http_status)s - %(status_label)s - Task id: %(task_id)s"

    api_version_bme_celery = 2

    ditmpl = {
        "task_params": {"task_id": undefined},
        "result": undefined,
        "api_version_bme_celery": undefined,
        "li_user_message": [{"message": undefined, "level": "info", "body": ""}],
    }

    def JsonResponse(self, *, messages: None | list[dict[str, Any] | str] = None, data=None):
        data = data or {}
        di_sub = dict(
            taskname=self.task.__name__,
            http_status=data.get("http_status") or self.DI_STATE_HTTP_STATUS.get(self.tmp.state, self.tmp.state),
            status_label=data.get("status_label") or self.tmp.state.lower(),
        )
        if isinstance(self.tmp.result, Exception):
            data["result"] = f"{self.tmp.result}[{type(self.tmp.result)}]"
        subber = Subber(di_sub, data, self.tmp, self.subber, self)
        di_sub["message"] = fill_template(self.tmpl_msg, subber)
        di = fill_di_template(self.ditmpl, subber, self)
        di["task_params"].update(**self.options.dict())

        if messages:  # 🏷 105fra.119sel.001sel.073.django_messages.eval
            if isinstance(messages, (str, dict)):
                messages = [messages]
            for message in messages:
                if isinstance(message, str):
                    message = dict(message=message, level="info", body="")
                di["li_user_message"].append(message)
        response = JsonResponse(di)
        http_status = di_sub["http_status"]
        if isinstance(http_status, int):
            response.status_code = http_status
        return response


class CeleryRunner(CeleryBaseRunner, MinimalManager):
    """Runs only the task from a request"""

    def __init__(self, request: Any, f_view=Callable, force_sync: bool = False, **kwargs):
        super().__init__(request=request, f_view=f_view, **kwargs)
        self.data = kwargs
        self.force_sync = force_sync


class BatchRunner(CeleryRunner):
    batchname: str


class BatchRunnerv3(CeleryRunner):
    # batchname: str

    batchconfig: ClassVar[BatchProxyConfig]
    rdbname: str


class BatchRunnerMix(CeleryBaseRunner):
    """provide celery run functionality to page"""


class BatchRunnerQuery(CeleryRunner):
    t_sql: str

    def run(self, *args, **kwargs):
        """the fetch actually runs the query."""
        # since
        msg = f"{self}.fetch:start"
        log(msg, loglevel="info")
        clogger = get_task_logger(f"{self}")
        clogger.info(msg)
        self.options = self.options_cls(**self.data)
        msg = f"{self}.fetch.{self.task=}.call"
        log(msg, loglevel="info")
        clogger.info(msg)
        fn = self.task.apply if self.force_sync else self.task.apply_async
        self.tmp = res = fn([self.t_sql, self.options.dict()])
        msg = f"{self}.fetch.{self.task=}.done"
        log(msg, loglevel="info")
        clogger.info(msg)
        if rpdb():  # pragma: no cover
            breakpoint()
            pass
        return res

    # old-style aliasing
    fetch = run


class MixViewShowing:
    """adds search info directly into di_settings"""

    searchoptions: BaseModel | None = None


if __name__ == "__main__":
    from bemyerp.lib.utils import set_rpdb

    rpdb = set_rpdb()

    class Fakey(VueManager2):
        def __init__(self, *args, **kwargs):
            self.f_view = None

        levp = "      "

        def debug(self, *args, **kwargs):
            "do-nothing"

        def call_subscribers(self, *args, **kwargs):
            print(f"{self.levp}🔬{self}.prep_all")

        def prep_all(self, *args, **kwargs):
            print(f"{self.levp}🔬{self}.call_subscribers")
            self.prep()
            self.dowork()
            self.do_integrity_checks()
            self.call_subscribers()
            self.finalize()
            self.post_finalize()

        def prep_html(self, main_template: str | None = None, persist=True, f_callback_pregen=None):
            print(f"  🔬{self}.prep_html calls self.prep_all")
            print(f"    🔬{self}.prep_all calls sub stuff")
            self.prep_all(persist, f_callback_pregen)
            return "<some html>"

        def prep(self, di_spike={}, *args, **kwargs):
            print(f"{self.levp}🔬{self}.prep")

        def fetch(self, *args, **kwargs):
            print(f"{self.levp}🔬{self}.fetch calls self.fetch2")
            self.fetch2(*args, **kwargs)

        def fetch2(self, *args, **kwargs):
            print(f"{self.levp}  🔬{self}.fetch2")

        def do_integrity_checks(self, *args, **kwargs):
            print(f"{self.levp}  🔬{self}.do_integrity_checks")

        def finalize(self):
            if rpdb():
                breakpoint()
            print(f"{self.levp}🔬{self}.finalize")

        def post_finalize(self):
            print(f"{self.levp}🔬{self}.post_finalize")

        def cleanup(self):  # Fakey
            print(f"{self.levp}🔬{self}.cleanup")

        def HttpResponse(self, main_template=None, persist=True, f_callback_pregen=None):
            print(f"🔬{self}.HttpResponse. calls self.prep_html")
            html = self.prep_html(main_template, persist, f_callback_pregen)
            return html

        def dowork(self):
            print(f"{self.levp}🔬{self}.dowork")

    fakey = Fakey()
    fakey.HttpResponse()
