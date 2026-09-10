"""descriptor/provider-based generic utility classes"""

import inspect
import os
import re
from dataclasses import dataclass
from dataclasses import field as dc_field
from functools import lru_cache
from operator import attrgetter
from typing import Any, Callable, Iterable, Type

from typing_extensions import Protocol

import bemyerp.pssystem.constants as constants
from bemyerp.lib._baseutils import AskedSub, ReturnVarSubstit
from bemyerp.lib.allowed_rdbnames import allowed_rdbnames
from bemyerp.lib.utils import (  # noqa - debug only
    Subber,
    _fmt_as_tuple_str,
    cprint,
    derive_objname,
    fill_template,
    nested_path_getter,
    pp,
    ppp,
    rin,
    rindebug,
    set_breakpoints3,
    set_cpdb,
    set_rpdb,
    tuple_to_yes_nos,
)
from bemyerp.pssystem.db import SysDb
from bemyerp.pssystem.dsl import EndpointTypes
from bemyerp.pssystem.exceptions import DataNotFoundException, InvalidConfigurationException
from bemyerp.pssystem.singletons import get_helperps, loader, urlregistry
from bemyerp.tests.helper_check_flagfile import check_flagfile
from bemyerp.websec.log_settings import Log

log = Log(__name__)


undefined = constants.undefined


def cpdb(*args, **kwargs):
    "disabled"


rpdb = breakpoints = cpdb


def get_args(fn):
    try:
        if hasattr(fn, "func_closure"):
            # does this only work for @login_required by itself?
            meta2 = inspect.signature(fn.__closure__[-1].cell_contents)
        else:
            meta2 = inspect.signature(fn)

        return meta2._parameters

    # pragma: no cover pylint: disable=unused-variable
    except (Exception,) as e:
        if cpdb():
            breakpoint()
        raise


class DHelperPSProvider:
    def __init__(self, rdbattrname: str = "rdbname"):
        self.rdbattrname = rdbattrname

    def __get__(self, obj, objtype=None):
        rdbname = getattr(obj, self.rdbattrname)
        res = get_helperps(rdbname)
        return res


class DRecnameProvider:
    def __get__(self, obj, objtype=None):
        try:
            subj = type(obj) if obj is not None else objtype

            res = derive_objname(subj.__name__)
            return res
        # pragma: no cover pylint: disable=unused-variable
        except (Exception,) as e:
            if cpdb():
                breakpoint()
            raise


class RecinfoProvider:
    def __get__(self, obj, objtype=None):
        try:
            recname = getattr(obj, "recname", None) or derive_objname(type(obj).__name__)

            if not recname.isupper():
                objecttype = getattr(obj, "objecttype", None)
                if objecttype is not None:
                    smartystub = constants.di_SmartObjectTypeStub.get(objecttype)
                    cand = getattr(smartystub, "recname", None)
                    if cand:
                        recname = cand

            res = get_psrecdefn(obj.rdbname, recname=recname, ps_or_pg=True)
            return res

        # pragma: no cover pylint: disable=unused-variable
        except (Exception,) as e:
            if cpdb():
                breakpoint()
            raise


def get_objecttype_from_context(context, rawvalue: Any) -> int:
    """try to deduce an objecttype from a context object"""
    try:
        if isinstance(rawvalue, str):
            if breakpoints("get_objecttype_from_context", {"rawvalue": rawvalue}):  # pragma: no cover
                # print(f"\n\n🔬🔬🔬rawvalue:{rawvalue}")
                breakpoint()
                pass

            if rawvalue.startswith("."):
                f = attrgetter(rawvalue[1:])
                rawvalue = f(context)

        if isinstance(rawvalue, int):
            return rawvalue
        else:
            # class DObjecttypeProvider: leverage
            rawvalue = rawvalue.split(".")[-1]
            recname = derive_objname(rawvalue)
            res = recname_to_objecttype(recname)
            return res
    # pragma: no cover pylint: disable=unused-variable
    except (Exception,) as e:
        if cpdb():
            breakpoint()
        raise


@lru_cache
def recname_to_objecttype(recname: str) -> int:
    try:
        import bemyerp.pssystem.constants as constants

        rows = constants._get_objecttypes()

        if not (di_mapping := getattr(recname_to_objecttype, "di_mapping", None)):
            di_mapping = dict(
                PRCSDEFN=constants.OBJECTTYPE_PROCESS_DEFINITION[0],
                PSPRDMDEFN=54,
                PSROLEDEFN=constants.OBJECTTYPE_ROLE[0],
                PSCLASSDEFN=constants.OBJECTTYPE_PERMISSION_LIST[0],
                Usergroup=constants.KOBJECTTYPE_USERGROUP[0],
                usergroup=constants.KOBJECTTYPE_USERGROUP[0],
                module=constants.KOBJECTTYPE_MODULE[0],
                PSPRSMDEFN=constants.OBJECTTYPE_PORTAL_REGISTRY_STRUCTURE[0],
                PSOPRDEFN=constants.KOBJECTTYPE_USER[0],
                remotedatabase=constants.KOBJECTTYPE_DATABASE[0],
            )

            with SysDb.get(initiator=f"{__name__}.recname_to_objecttype") as sysdb:
                rows = sysdb.select("select objecttype, recname from pssystem_objecttype")
                di = {row.recname: row.objecttype for row in rows}
                di_mapping |= di

            recname_to_objecttype.di_mapping = di_mapping

        return di_mapping[recname]

    # pragma: no cover pylint: disable=unused-variable
    except (Exception,) as e:
        if cpdb():
            breakpoint()
        raise


class DObjecttypeProvider:
    "determine objecttype on the fly"

    def __init__(self, attrname=None):
        self.attrname = attrname

    _di_mapping = undefined

    @property
    def di_mapping(self):
        try:
            cls = type(self)
            if cls._di_mapping is undefined:
                import bemyerp.pssystem.constants as constants

                di_mapping = dict(
                    PRCSDEFN=constants.OBJECTTYPE_PROCESS_DEFINITION[0],
                    PSPRDMDEFN=54,
                    PSROLEDEFN=constants.OBJECTTYPE_ROLE[0],
                    PSCLASSDEFN=constants.OBJECTTYPE_PERMISSION_LIST[0],
                    Usergroup=constants.KOBJECTTYPE_USERGROUP[0],
                    usergroup=constants.KOBJECTTYPE_USERGROUP[0],
                    module=constants.KOBJECTTYPE_MODULE[0],
                    PSPRSMDEFN=constants.OBJECTTYPE_PORTAL_REGISTRY_STRUCTURE[0],
                )
                cls._di_mapping = di_mapping
            return cls._di_mapping
        # pragma: no cover pylint: disable=unused-variable
        except (Exception,) as e:
            if cpdb():
                breakpoint()
            raise

    def __get__(self, obj, objtype=None) -> int:
        try:
            if not self.attrname:
                name = type(obj).__name__
            else:
                name = getattr(obj, self.attrname)

            recname = derive_objname(name)
            res = recname_to_objecttype(recname)
            return res
        # pragma: no cover pylint: disable=unused-variable
        except (KeyError,) as e:
            msg = f"could not determine objecttype for {name}.  known : {'\n '.join(self.di_mapping.keys())}"

            new_exc = InvalidConfigurationException(msg)
            log(msg, loglevel="exception")
            if cpdb():
                breakpoint()
                pass
            raise new_exc

        # pragma: no cover pylint: disable=unused-variable
        except (Exception,) as e:
            if cpdb():
                breakpoint()
            raise


class DNestedProvider:
    "get value from deep in data structures"

    def __init__(self, datapath, default: Any = undefined):
        self.datapath = datapath
        self.default = default

    def __get__(self, inst, objtype=None) -> Any:
        try:
            try:
                return nested_path_getter(inst, self.datapath)
            # pragma: no cover pylint: disable=unused-variable
            except (
                KeyError,
                AttributeError,
            ) as e:
                if self.default is not undefined:
                    if fcopy := getattr(self.default, "copy", None):
                        return fcopy()
                    return self.default
                raise

        # pragma: no cover pylint: disable=unused-variable
        except (
            KeyError,
            AttributeError,
        ) as e:
            raise
        except (Exception,) as e:
            if cpdb():
                breakpoint()
            raise


class DTstFuncArgsProvider:
    def __init__(self, dottedfunc: str, wanteds: Iterable[str] | str):
        self.dottedfunc = dottedfunc

        if isinstance(wanteds, str):
            wanteds = wanteds.split()

        self.wanteds = tuple(wanteds)

    def __set_name__(self, owner, name):
        self.name = name

    def __get__(self, inst, objtype=None) -> dict[str]:
        """calculate function arguments and return as dict"""
        try:
            classname = type(inst).__name__

            if breakpoints("DProviderArgs", {"classname": classname}):  # pragma: no cover
                # print(f"\n\n🔬🔬🔬classname:{classname}")
                breakpoint()
                pass

            srcname, path_ = self.dottedfunc.split(".", maxsplit=1)

            if not srcname == "inst":
                raise NotImplementedError

            fn = nested_path_getter(inst, path_)
            argnames = get_args(fn)
            di_care = {k: v for k, v in argnames.items() if k in self.wanteds}

            res = {}
            for attrname in di_care.keys():
                res[attrname] = getattr(inst, attrname, undefined)

            return res
        # pragma: no cover pylint: disable=unused-variable
        except (Exception,) as e:
            if cpdb():
                breakpoint()
            raise


class DUiObjP_URLProvider:
    """get the url component for the objecttype at hand:
    ex: 53/PSCLASSDEFN shows up as `permissions` in the urls.

    This is configured in either psconstants.py or constants.py

    """

    def __get__(self, inst, objtype=None):
        try:
            objecttype = inst.objecttype
            config = constants.calc_objecttypes(constants)[objecttype]
            res = config.p_url
            if not res:
                raise InvalidConfigurationException(f"{self}.__get__({inst=}).  No `p_url` configured for {objecttype=}")

            return res
        # pragma: no cover pylint: disable=unused-variable
        except (Exception,) as e:
            if cpdb():
                breakpoint()
            raise


class DUseV2Checker:
    def __get__(self, obj, objtype=None):
        return check_flagfile()


class DPsrecdefnProvider:
    def __init__(self, attr_rdbname="rdbname", attr_recname="recname"):
        self.attr_rbname = attr_rdbname
        self.attr_recname = attr_recname

    def __set_name__(self, owner, name):
        self.name = name

    def __get__(self, inst, objtype=None):
        try:
            helperps = get_helperps(getattr(inst, self.attr_rbname))

            recname = getattr(inst, self.attr_recname)
            psrecdefn = helperps.get_table(recname)
            return psrecdefn
        # pragma: no cover pylint: disable=unused-variable
        except (Exception,) as e:
            log(f"DPsrecdefnProvider@{self.name}.__get__:exc={e}", loglevel="exception")
            if cpdb():
                breakpoint()
            raise


@lru_cache
def get_psrecdefn(rdbname: str, recname: str, ps_or_pg=False, is_pg: bool | None = None):
    "supports either getting the regular PSRECDEFN or postgres bme ones (assumed to be from sysdb)"
    try:
        if not recname:
            raise InvalidConfigurationException(f"get_psrecdefn:{recname=}:")

        helperps = get_helperps(rdbname)

        # hack, but necessary for some cases where we come in from `derive_objname`

        recname = {"usergroup": "bme_usergroup"}.get(recname.lower(), recname)

        psrecdefn = helperps.get_table(recname)

        # 🏷 100fun038pssfp002pso004urldbpgfin92securityusersajameserror - another branch error point
        if not psrecdefn.IS_PS:
            psrecdefn = helperps.get_pg_record(recname)
        elif ps_or_pg:
            # can we actually get a record definition?
            try:
                _ = psrecdefn.psrecdefn_rdb
            # pragma: no cover pylint: disable=unused-variable
            except (DataNotFoundException,) as e:
                psrecdefn = helperps.get_pg_record(recname)

        return psrecdefn
    # pragma: no cover pylint: disable=unused-variable
    except (Exception,) as e:
        if cpdb():
            breakpoint()
        raise


class DObjtypeInfoProvider:
    def __get__(self, obj, objtype=None):
        calculated_objecttypes = loader.calculated_objecttypes
        value = calculated_objecttypes[obj.objecttype]

        return value


@lru_cache
def get_recinfo_by_objecttype(rdbname: str, objecttype: int):
    try:
        if objecttype < 1000:
            is_pg = False
            recname = constants.di_objecttype_recname[objecttype]
        else:
            is_pg = True
            bmeobjtypeinfo = constants.calc_objecttypes(constants)[objecttype]
            recname = bmeobjtypeinfo.recname

        # PSOPRDEFN
        if objecttype in (1002,):
            is_pg = False

        recinfo = get_psrecdefn(rdbname=rdbname, recname=recname, ps_or_pg=True, is_pg=is_pg)
        return recinfo

    # pragma: no cover pylint: disable=unused-variable
    except (Exception,) as e:
        if cpdb():
            breakpoint()
        raise


class DRdbnamesFromRequestProvider:
    def __repr__(self) -> str:
        return f"{self.__class__.__name__}"

    def __get__(self, obj, objtype=None):
        try:
            prefix = f"{self}:"
            subj = type(obj) if obj is not None else objtype
            request = getattr(obj, "request", None)
            if request is None:
                return []

            res = request.session.get("li_rdbname")
            # log(f"{prefix}470:{res=}", loglevel="warning")
            if res is None:
                res = allowed_rdbnames(obj.request.user.username, check_live=True)
                obj.request.session["li_rdbname"] = res.copy()

            rdbname = obj.rdbname
            res2 = [rdbname] + [v for v in res if not v == rdbname]
            # log(f"{prefix}.li_rdbname=>{res2}", loglevel="warning")
            return res2
        # pragma: no cover pylint: disable=unused-variable
        except (Exception,) as e:
            if os.getenv("cpdb") or cpdb():
                breakpoint()
            raise


###################
# UrlProviders, latest approach
###################


@dataclass
class _ProviderUrlSpec:
    # constants.appname_resolver.getappnameurl
    resolve: str = ""


class DurlProviderFillTemplater(Protocol):
    """custom url provider protocol"""

    def fill_template(self, vmgr: Any, spec: Any, t_url: str):
        "do-nothing url template filler method signature"


@dataclass(kw_only=True)
class _BaseProviderUrlSpec:
    resolve: str = ""
    fill_templater: DurlProviderFillTemplater | None = None

    # used to do things like discerning cmp0/cmp1
    key: str = ""


@dataclass(kw_only=True)
class ProviderUrlSpecDirect(_BaseProviderUrlSpec):
    # t_url: str
    resolve: str = ""
    aliases: dict[str, str] = dc_field(default_factory=dict)


@dataclass(kw_only=True)
class ProviderUrlSpecLookup(_BaseProviderUrlSpec):
    lookup: str
    aliases: dict[str, str] = dc_field(default_factory=dict)


@dataclass(kw_only=True)
class _BaseInternalProviderUrlSpec: ...


@dataclass(kw_only=True)
class _InternalProviderUrlSpec(_BaseInternalProviderUrlSpec):
    """when we know everything without seeing an instance of the vmgr"""

    no_resolve: dict[str, str]
    urlname: str
    t_url: str

    def __post_init__(self):
        try:
            assert isinstance(self.no_resolve, dict)
        except (Exception,) as e:
            if cpdb():
                breakpoint()
                pass
            raise

    def fill_template(self, vmgr):
        try:
            return fill_template(self.t_url, self.no_resolve, vmgr.subber)
        except (Exception,) as e:
            if cpdb():
                breakpoint()
                pass
            raise


@dataclass(kw_only=True)
class _DynamicInternalProviderUrlSpec(_BaseInternalProviderUrlSpec):
    """when we need a vmgr instance to work with"""

    spec: ProviderUrlSpecLookup
    provider: Any

    def __post_init__(self):
        assert self.spec, f"{self}.spec={self.spec}"

    def fill_template(self, vmgr):
        try:
            tmp = {}
            t_lookup = self.spec.lookup
            if "%(objname)s" in t_lookup:
                tmp["objname"] = derive_objname(vmgr).lower()

            lookup = fill_template(t_lookup, tmp, vmgr)

            t_url = urlregistry.get(lookup)

            if self.spec.fill_templater:
                res = self.spec.fill_templater.fill_template(vmgr=vmgr, spec=self.spec, t_url=t_url)
                return res

            # do we need to worry about aliases?
            if self.spec.aliases:
                di_alias = {k: f"%({v})s" for k, v in self.spec.aliases.items()}

                t_url = fill_template(t_url, di_alias, ReturnVarSubstit())

            # see what's in the template and rule out what weren't asked to resolve
            generics = self.provider.resolve or ""
            specifics = ""
            if self.spec:
                specifics = self.spec.resolve

            _, nos = resolve_yes_no(template=t_url, generic_resolve=generics, specific_resolve=specifics)

            no_resolve = {k: f"%({k})s" for k in nos}

            res = fill_template(t_url, no_resolve, vmgr.subber)

            return res
        except (Exception,) as e:
            if cpdb():
                breakpoint()
                pass
            raise


class InstUrlProvider:
    def __init__(self, vmgr: Any, provider: "DUrlProvider"):
        self.vmgr = vmgr
        self.provider = provider

    @classmethod
    def internal_factory(
        cls,
        t_url: str,
        provider: Any,
        spec: ProviderUrlSpecLookup,
        lookup: str = "",
        vmgr_templated=False,
    ) -> _BaseInternalProviderUrlSpec:
        """
        t_url = "%(foo)s/%(bar)s", resolves = {"foo"}
        means returning {"bar" : "%(bar)s"} so that bar is NOT resolved later on.
        """
        try:
            # see what's in the template and rule out what weren't asked to resolve
            askedsub = AskedSub()
            _ = t_url % askedsub

            generics = provider.resolve or ""
            specifics = ""
            if spec:
                specifics = spec.resolve

            _, nos = resolve_yes_no(template=t_url, generic_resolve=generics, specific_resolve=specifics)

            no_resolve = {k: f"%({k})s" for k in nos}
            res = _InternalProviderUrlSpec(urlname=lookup, no_resolve=no_resolve, t_url=t_url)
            return res
        except (Exception,) as e:
            if os.getenv("cpdb") or cpdb():
                breakpoint()
                pass
            raise

    @classmethod
    def _via_lookup(cls, lookup: str, provider: "DUrlProvider", spec: Any) -> _InternalProviderUrlSpec:
        try:
            t_url = urlregistry.get(lookup)
            res = cls.internal_factory(t_url, lookup=lookup, provider=provider, spec=spec)
            return res
        except (Exception,) as e:
            if os.getenv("cpdb") or cpdb():
                breakpoint()
                pass
            raise

    # @classmethod
    #     try:
    #         if spec_resolves:
    #             return tmp

    #         return resolves_from_provider
    #             pass
    #         raise

    @classmethod
    def build_internal_spec(cls, urlkey: str, spec: str | _ProviderUrlSpec | None, provider: "DUrlProvider") -> _InternalProviderUrlSpec:
        try:
            prefix = f"InstUrlProvider@{urlkey=}"
            resolves = provider.resolve

            if breakpoints("build_internal_spec", {"urlkey": urlkey}):  # pragma: no cover
                breakpoint()
                pass

            #

            t_url = None

            if spec is None:
                """psprsmdefn_detail=None:
                This is by definition a lookup using the urlkey and appname
                so lookup is "security:psprsmdefn_detail"
                and then figure out the no-resolves against the found template, `/db/%(rdbname)s/security/portal/%(CREF)s/`
                which would typically means `{"CREF": "%(CREF)s"}` given a resolve of `{"rdbname"}`
                meaning that client-side JS would have a template like `/db/HCM92ORP/security/portal/%(CREF)s/`

                """
                lookup = f"{provider.appname_for_url_lookup}:{urlkey}"
                return cls._via_lookup(lookup, provider=provider, spec=None)

            if isinstance(spec, str):
                """two flavors - either a direct template string with %(<vars>)s or a simple lookup"""
                if "%" in spec:
                    res = cls.internal_factory(t_url=spec, resolves=resolves, provider=provider, spec=None)
                    return res
                res = cls._via_lookup(lookup=spec, provider=provider, spec=None)
                return res

            if isinstance(spec, ProviderUrlSpecLookup):
                if "%" in spec.lookup:
                    res = _DynamicInternalProviderUrlSpec(spec=spec, provider=provider)
                    return res
                else:
                    res = cls._via_lookup(spec.lookup, provider=provider, spec=spec)
                    return res

        except (Exception,) as e:
            if os.getenv("cpdb") or cpdb():
                breakpoint()
                pass
            raise

    def __getattr__(self, urlname) -> str:
        try:
            vmgr = self.vmgr
            spec = self.provider.urls[urlname]
            url = spec.fill_template(vmgr)
            return url
        except (Exception,) as e:
            #     pass
            raise


@lru_cache
def endpointtype_by_name(name: str) -> EndpointTypes:
    try:
        name = name.lower()
        if "detail" in name:
            return EndpointTypes.detail
        if "compare" in name:
            return EndpointTypes.compare
        if "xdb" in name:
            return EndpointTypes.xdb
        return EndpointTypes.other

    except (Exception,) as e:
        if cpdb():
            breakpoint()
            pass
        raise


class DEndpointTypeProvider:
    "am I a Detail, Compare, XDB, Batch?"

    def __get__(self, obj, objtype=None) -> EndpointTypes:
        try:
            objtype = objtype or type(obj)
            name = objtype.__name__

            return endpointtype_by_name(name)

        except (Exception,) as e:
            if cpdb():
                breakpoint()
                pass
            raise


@lru_cache
def resolve_yes_no(template: str, generic_resolve: str = "", specific_resolve: str = ""):
    try:
        generics = _fmt_as_tuple_str(generic_resolve)
        specifics = _fmt_as_tuple_str(specific_resolve)

        yes_no_generics = tuple_to_yes_nos(generics)

        yes_no_specifics = tuple_to_yes_nos(specifics)

        gen_yes = [re.compile(pat) for pat in yes_no_generics.yeses]
        gen_nos = [re.compile(pat) for pat in yes_no_generics.noses]

        res = set()

        spec_yes = [re.compile(pat) for pat in yes_no_specifics.yeses]
        spec_nos = [re.compile(pat) for pat in yes_no_specifics.noses]

        askeds = set(AskedSub.find_varnames(template))

        # first dibs for the generics, add stuff
        for varname in askeds:
            if any(patre.match(varname) for patre in gen_yes):
                res.add(varname)

        # then remove it
        for varname in list(res):
            if any(patre.match(varname) for patre in gen_nos):
                if varname in res:
                    try:
                        res.remove(varname)
                    except (KeyError,) as e:
                        pass

        # specific adds win
        for varname in askeds:
            if any(patre.match(varname) for patre in spec_yes):
                res.add(varname)

        # unless countered by a specific remove
        for varname in list(res):
            if any(patre.match(varname) for patre in spec_nos):
                if varname in res:
                    try:
                        res.remove(varname)
                    except (KeyError,) as e:
                        pass

        resno = askeds - res

        return (res, resno)
    # pragma: no cover pylint: disable=unused-variable
    except (Exception,) as e:
        if cpdb():
            breakpoint()
        raise


class DUrlProvider:
    def __init__(self, appname: str, urls: dict[str, str | _ProviderUrlSpec | None], resolve: str = ""):
        self.resolve = resolve
        self.appname_for_url_lookup = constants.appnameresolver.getappnameurl(appname)
        self.urls = {k: InstUrlProvider.build_internal_spec(k, spec=v, provider=self) for k, v in urls.items()}

    def __get__(self, obj, objtype=None):
        try:
            return InstUrlProvider(vmgr=obj, provider=self)
        except (Exception,) as e:
            if cpdb():
                breakpoint()
                pass
            raise

    def calc_resolves(self, resolve: str | tuple[str], t_url: str | None = None) -> set[str]:
        try:
            raise NotImplementedError()
            if not resolve:
                return self.resolve

            if "*" in str(resolve):
                breakpoint()
                pass

            if isinstance(resolve, str):
                resolve = _fmt_as_tuple_str(resolve)

            resolve = tuple(resolve)
            yes_no = tuple_to_yes_nos(resolve)

            res = self.resolve | set(yes_no.yeses)

            res = res - set(yes_no.noses)

            return res

        except (Exception,) as e:
            if cpdb():
                breakpoint()
                pass
            raise


class DProviderObjectvalue:
    def __set_name__(self, owner, name):
        self.name = name

    def __get__(self, vmgr, objtype=None):
        try:
            if vmgr is None:
                return undefined
            keys = vmgr.recinfo.li_key_fieldname
            ix = int(self.name[-1]) - 1
            keyname = keys[ix]
            return vmgr.fetcher_criteria[keyname]
        # pragma: no cover pylint: disable=unused-variable
        except (Exception,) as e:
            log(f"DPsrecdefnProvider@{self.name}.__get__:exc={e}", loglevel="exception")
            if cpdb():
                breakpoint()
            raise


class DWriteOnceProvider:
    """only allows the attribute to be set once"""

    def __set_name__(self, owner, name):
        self.name = name
        self.private_name = f"_{name}"

    def __get__(self, obj, objtype=None):
        value = getattr(obj, self.private_name, undefined)
        return value

    def __set__(self, obj, value):
        try:
            # log(f"{type(self).__name__}.{self.name}.set:{id(value)=}", loglevel="warning")
            old_value = self.__get__(obj)
            if old_value is undefined:
                setattr(obj, self.private_name, value)
            else:
                raise ValueError(f"{type(obj).__name__}.{self.name} is already set to {old_value}.  can't set to {value}")
        except (Exception,) as e:
            log(f"DPsrecdefnProvider@{self.name}.__get__:exc={e}", loglevel="exception")
            if cpdb():
                breakpoint()
            raise


class DAutoBuildProvider:
    """can autoprovide a particular class against its obj/owner
    cls_         : the class to build, defaults to `dict`
    f_get_kwargs : a function that returns a dict from the owner to pass
                   to cls_'s __init__

    its `__set__` also blocks later assignments to the variable

    ex:
        context = DAutoBuildProvider(cls_ = PreContext, f_get_kwargs=lambda vmgr: {"vmgr": vmgr})

        same as vmgr.context = PreContext(vmgr=vmgr)
    """

    def __set_name__(self, owner, name):
        self.name = name
        self.private_name = f"_{name}"

    def __init__(self, cls_: Type = dict, f_get_kwargs: Callable | None = None):
        self.cls_ = cls_
        self._get_kwargs = f_get_kwargs

    def __get__(self, obj, objtype=None):
        ...
        if obj is None:
            return None

        if value := getattr(obj, self.private_name, None):
            return value

        kwargs_ = {}

        if self._get_kwargs:
            kwargs_ = self._get_kwargs(obj)

        value = self.cls_(**kwargs_)
        setattr(obj, self.private_name, value)
        return value

    def __set__(self, obj, value):
        raise AttributeError(f"Cannot set '{self.name}' directly")


class MROConfigSubberProvider:
    """
    given a datapath for a config dictionary or object this will grab each entry that concerns a classname
    in the mro and builds a Subber from it.

    example:
    ````

    yaml config:
        reports:
            Foo:
                header : "hfoo"
            Bar:
                header : "hzoom"
                footer : "fzoom"

    class Foo(Bar):

        config = MROConfigSubberProvider(datapath="_config.reports")

        def __init__(self):
            self.config = ... #(loaded from yaml, somehow)
            self.config.get("header") # -> "hfoo"
            self.config.get("footer") # -> "fzoom"
            self.config.get("title","unknown") # -> "unknown"
    ````
    """

    def __init__(self, datapath: str):
        self.datapath = datapath

    def __get__(self, obj, objtype=None) -> Subber:
        try:
            di_config_all_classnames = nested_path_getter(obj, self.datapath)
            anc_classnames = [t.__name__ for t in objtype.mro()]
            li_conf = [cconf for classname in anc_classnames if (cconf := di_config_all_classnames.get(classname))]
            res = Subber(*li_conf)
            return res
        # pragma: no cover pylint: disable=unused-variable
        except (Exception,) as e:
            if cpdb():
                breakpoint()
            raise
