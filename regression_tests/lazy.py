#!/usr/bin/env python
"""lazy testing for rxedit
"""
import json
import os
import subprocess
import sys
import tomllib
import unittest
from dataclasses import dataclass
from logging.config import dictConfig
from pathlib import Path
from types import SimpleNamespace
from typing import Any

from yaml import safe_load as yload

from lazy_regression_tests._baseutils import parse_dict_to_simplenamespaces, read_data
from lazy_regression_tests.basics import lazyinfo
from lazy_regression_tests.common import ExtAlias, get_lazy_logger
from lazy_regression_tests.core_assist2 import Checker, LazyRoot, _LazyMeta, undefined
from lazy_regression_tests.utils import InvalidConfigurationException, set_breakpoints3, set_cpdb, set_rpdb

logger = get_lazy_logger()

def configure_lazy_logging(log_file: str|Path|None = None):
    """Configure handlers for the shared `lazy` logger used by lazy_regression_tests."""

    log_file = log_file or os.getenv("fnp_lazy_log", Path(os.getenv("lzrt_info_dir",".")) / "lazy.log")


    pa_log = Path(log_file).expanduser().resolve()
    pa_log.parent.mkdir(parents=True, exist_ok=True)


    # Keep config app-owned: only the logger name is shared with the library.
    dictConfig(
        {
            "version": 1,
            "disable_existing_loggers": False,
            "formatters": {
                "standard": {
                    "format": "%(asctime)s %(levelname)s [%(name)s] %(message)s",
                }
            },
            "handlers": {
                "lazy_file": {
                    "class": "logging.FileHandler",
                    "level": "WARNING",
                    "formatter": "standard",
                    "filename": str(pa_log),
                    "mode": "w",
                    "encoding": "utf-8",
                },
                "lazy_errors": {
                    "class": "logging.StreamHandler",
                    "level": "ERROR",
                    "formatter": "standard",
                    "stream": "ext://sys.stderr",
                },
            },
            "loggers": {
                "lazy": {
                    "level": "WARNING",
                    "handlers": ["lazy_file", "lazy_errors"],
                    "propagate": False,
                }
            },
        }
    )


configure_lazy_logging()



#################################################################
# lazy stuff
#################################################################


class CustomLazyMixin:
    """we'll see what we actually need """

    def __repr__(self) -> str:
        return f"{self.__class__.__name__}"

    def lazyinfo(self, *args, **kwargs):
        lazyinfo(*args, **kwargs)


class JDataProvDescriptor:  # Config file provider
    """loads config files"""

    def __repr__(self) -> str:
        return f"{self.__class__.__name__}"

    def __init__(
        self,
        src: Any,
        as_simplenamespace=False,
        attrname_dict_backup: str | None = None,
        cache=True,
        subdomain: str = "",
    ):
        self.src = src
        self.as_simplenamespace = as_simplenamespace
        self.attrname_dict_backup = attrname_dict_backup
        self.cache = cache
        self.subdomain = subdomain

    def __set_name__(self, owner, name):
        self.name = name
        self.cachename = f"_{name}"

    def _parse_dict_to_simplenamespaces(self, data) -> SimpleNamespace:
        try:
            res = parse_dict_to_simplenamespaces(data)

            if self.attrname_dict_backup:
                setattr(self.tmpinstance, self.attrname_dict_backup, data)

            return res
        # pragma: no cover
        except (Exception,) as _e:
            if cpdb():
                breakpoint()
            raise

    def _cache_and_return(self, data):
        try:
            if self.subdomain:
                data = data[self.subdomain]
            if self.as_simplenamespace:
                data = self._parse_dict_to_simplenamespaces(data)
            if self.cache:
                setattr(self.tmpinstance, self.cachename, data)
            return data
        # pragma: no cover
        except (Exception,) as _e:
            if cpdb():
                breakpoint()
            raise

    def __get__(self, instance, owner) -> dict[str, Any]:
        "load json, somehow"
        try:
            try:
                lookfor = self.src
                if (v := getattr(instance, self.cachename, undefined)) is not undefined:
                    return v

                self.tmpinstance = instance

                if isinstance(lookfor, str) and lookfor.startswith("<") and lookfor.endswith(">"):
                    lookfor = getattr(instance, lookfor[1:-1])

                if isinstance(lookfor, Path):
                    j_data = read_data(lookfor)
                    return self._cache_and_return(j_data)

                elif isinstance(lookfor, dict):
                    res = lookfor.copy()
                    return self._cache_and_return(res)

                elif isinstance(lookfor, (str, Path)):
                    if str(lookfor).startswith("$"):  # environment variable
                        lookfor = os.getenv(str(lookfor)[1:])
                        if lookfor is None:
                            raise InvalidConfigurationException(f"Environment variable {lookfor} is not set")

                    if lookfor.startswith("{") or lookfor.startswith("["):
                        res = json.loads(lookfor)
                        return self._cache_and_return(res)

                    else:
                        pa = Path(lookfor)

                        suffix = pa.suffix
                        if suffix == ".json":
                            with pa.open() as fi:
                                res = json.load(fi)
                        elif suffix == ".toml":
                            with pa.open("rb") as fi:
                                res = tomllib.load(fi)
                        elif suffix == ".yaml":
                            with pa.open() as fi:
                                res = yload(fi)

                        else:
                            raise NotImplementedError(f"lookfor: {lookfor}")

                        return self._cache_and_return(res)

                else:
                    raise NotImplementedError(f"lookfor: {lookfor}")

            except (FileNotFoundError, NotImplementedError) as _e:
                if cpdb():
                    breakpoint()
                    pass
                raise

            # pragma: no cover
            except (Exception,) as _e:
                if cpdb():
                    breakpoint()
                raise
        # pragma: no cover
        except (Exception,) as _e:
            if cpdb():
                breakpoint()
            raise



VERBOSE = "-v" in sys.argv

def check_keep(suffix: str) -> tuple[bool, str]:
    """conditional activation of tests - RUN TEST ONLY IF"""

    envname = f"rxedit_test_{suffix}".replace("__", "_")

    res = bool(os.getenv(envname))
    msg = f"add ${envname}"
    if VERBOSE:
        print(msg, file=sys.stderr) # debugging
    return (res, msg)


def cpdb(*args, **kwargs):
    "disabled conditional breakpoints - does nothing until activated by set_cpdb/rpdb/breakpoint3"

rpdb = breakpoints = cpdb

if __name__ == "__main__":
   cpdb = set_cpdb()
   rpdb = set_rpdb()
   breakpoints = set_breakpoints3() or breakpoints


PA_SCRIPT = Path(__file__).expanduser().absolute().resolve()

class MixinBaseLazy(metaclass=_LazyMeta):
    "basic generic configuration for the base class"

    lz: LazyRoot

    cls_validators = dict(
        out = [ExtAlias(alias="txt")]
    )
    cls_filters = dict(
        out = [ExtAlias(alias="txt")]
    )

    @property
    def checker(self) -> Checker:
        try:
            return getattr(self.lz, self.ext)
        # pragma: no cover
        except (Exception,) as _e:
            if cpdb():
                breakpoint()
            raise


class LazyMixinScript:
    """
    pickup attributes from global scope
    """
    lazy_filename = PA_SCRIPT.stem


class Base(
    LazyMixinScript,
    CustomLazyMixin,
    MixinBaseLazy,
    unittest.TestCase):
    "base class"

    def _is_tst(self):
        if self.__class__ in TU_BASE_CLASSES:
            return False


class co:
    file_path = "file_path"
    commands = "commands"
    pa_source = "pa_source"


@dataclass
class Config:
    pa_source : Path
    commands: list[str]
    env: dict[str,str]


class Best_(Base):
    """base test"""

    _rawconfig = JDataProvDescriptor(PA_SCRIPT.with_suffix(".yaml"))

    config : Config

    shared_lib : str|None = None

    rxedit_plugins_directory : Path|None = None

    def setUp(self):
        try:
            if type(self) in TU_BASE_CLASSES:
                raise unittest.SkipTest("...base...")
            super().setUp()

            rawconfig = self._rawconfig[type(self).__name__]


            if self.shared_lib:
                if dn := os.getenv("rxedit_plugins_directory"):
                    pad_plugins = Path(dn).expanduser().resolve()
                    self.rxedit_plugins_directory = pad_plugins
                    if not pad_plugins.exists():
                        raise unittest.SkipTest(f"{pad_plugins=} doesn't exist")


                    if sys.platform == "darwin":
                        so_ext = "dylib"
                    elif os.name == "nt":
                        so_ext = "dll"
                    else:
                        so_ext = "so"

                    pa_shared = pad_plugins / f"librxedit_language_{self.shared_lib}.{so_ext}"
                    if not pa_shared.exists():
                        raise unittest.SkipTest(f"{pa_shared} shared library not found")
                else:
                    raise unittest.SkipTest(f"no env.$rxedit_plugins_directory specified")



            # bit of a hack to avoid choking on
            """
               commands:
                   - d::
                   ...
            """
            commands = []
            for command in rawconfig["commands"]:
                if isinstance(command,dict) and len(command) == 1:
                    k, v = next(iter(command.items()))
                    if k.endswith(":") and v is None:
                        command = k.rstrip(":")
                        commands.append(command)
                    else:
                        commands.append(command)
                    pass
                else:
                    commands.append(command)

            rawconfig["commands"] = commands
            if "env" not in rawconfig:
                rawconfig["env"] = {}

            file_path = rawconfig.pop(co.file_path) or getattr(self.file_path,None)
            if "$" in file_path:
                for envname in "srcdir_rs bme_home _diroiexplore diroisamples".split():
                    to_ = os.environ[envname]
                    from_ = f"${envname}"
                    file_path = file_path.replace(from_,to_)

            pa = Path(file_path)
            assert pa.exists(), f"{pa} not found"
            self.config = Config(**rawconfig, pa_source=pa)


        #pragma: no cover
        except (Exception,) as _e:
            if cpdb():
                breakpoint()
            raise

    rxedit_line_number = "0"

    def rxedit(self):
        try:
            classname = type(self).__name__
            li_arg = [os.environ["fnp_rxedit"]] + [str(self.config.pa_source)] + self.config.commands

            # rxedit_line_number
            # turn off line numbers to
            env = self.config.env.copy()

            if self.rxedit_plugins_directory:
                env.update(rxedit_plugins_directory=self.rxedit_plugins_directory)

            env.update(rxedit_line_number=self.rxedit_line_number,RUST_LOG="debug")

            env = {str(k):str(v) for k,v in sorted(env.items())}

            result = subprocess.run(li_arg, capture_output=True, text=True,env=env)
            lines = result.stdout.splitlines()
            _ = env.pop("RUST_LOG","")
            if env["rxedit_line_number"] == "0":
                _ = env.pop("rxedit_line_number","")

            s_env = f"env={str(env)}" if env else ""


            command = " ".join(["#"] + [self.config.pa_source.name] + li_arg[2:] + [s_env] ).strip()
            res = "\n".join([command] + lines)

            pad_stderr = Path(os.environ["lzrt_info_dir"]) / "rxedit.log"

            pad_stderr.mkdir(parents=True,exist_ok=True)

            pa_stderr = pad_stderr / f"{classname}.rxedit.err"

            with pa_stderr.open("w") as fo:
                fo.write(result.stderr)


            if breakpoints('rxedit', {'classname': classname}):  # pragma: no cover
                print(f'\n\n!!!rxedit.classname:{classname}:\n  {command}') # debugging
                breakpoint() # debugging
                pass # debugging

            return res
        #pragma: no cover
        except (Exception,) as _e:
            if cpdb():
                breakpoint()
            raise


    def test_it(self):
        """test cross-database (xdb) comparisons"""

        if type(self) in TU_BASE_CLASSES:
            raise unittest.SkipTest("...base...")

        response = None

        try:
            checker: Checker = self._lz.out
            response = self.rxedit()
            checker.assert_same(response)

            if breakpoints("test_stop_always", {"always": 1}):
                breakpoint()
                pass
            lazyinfo(response, testee=self)
        except (Exception,) as _e:  # pragma: no cover

            self.lazyinfo(response, testee=self, exception=_e)
            if cpdb():
                breakpoint()
            raise

class Test_Mainrs_Declare(Best_):
    "main.rs declaration"

class Test_Mainrs_DeclareNumbers(Best_):
    "main.rs declaration with -n"

class Test_Mainrs_Printdebugs(Best_):
    "main.rs declaration with -G and print! debug!"

class Test_PyViewManagerDescrA5(Best_):
    "anything with manager anywhere in the signature"


class Test_PyViewManagerMixBody(Best_):
    "class ...Mix... with body"

class Test_PySampleCAnd(Best_):
    "test declare:: followed by and::<pattern>"

class Test_PySampleCAndAfter2(Best_):
    "testing `declare:: and::add::A2` which should keep only signatures with `add` and then show 2 after"

    """  Issue.104rus001xed015commandandnowork

def add(self, a: int, b: int) -> int:
        Add two numbers  .
        result_add = a + b
        self.history.append(f"{a} + {b} = {result_add}")
        return result_add

    def multiply(self,
    calc.multiply(4, 7)
    calc.show_history()


    """

class Test_BadPySampleDeclareOr(Best_):
    """bad results out of 'd::mul|add::' """

class Test_DeclarationMultiLine(Best_):
    """multiline signature declaration:: support """

class Test_DeclarationBodyOwner(Best_):
    """ check d::init::bo shows both owner and body"""

class Test_DeclarationAfterBefore(Best_):
    """ check d::init::A1B2 """

class Test_RustSampleDeclareImpl(Best_):
    """impl-only lines are being missed

    impl Greeter for HelloApp {  missing
        fn greet(&self, name: &str) -> String {

    """
class  Test_DeclarationSetKey(Best_):
    """ set key then less on wk"""

class Test_DeclarationSetKeyEnd(Best_):
    "same but with string-end syntax `d::mul::sk=fo`"


class Test_Head3(Best_):
    "more first 3 lines"

class Test_From30(Best_):
    "more from 30+ lines"



class  Test_MoreShowOwner(Best_):
    """x pysample.py m::append::o"""


class  Test_Declare_ParentPath(Best_):
    ...

class  Test_Declare_Parented(Best_):
    ...

class  Test_Declare_ChildrenOf(Best_):
    ...

class Test_Declare_Negation(Best_):
    """use name search negation foo--bar style"""

class Test_Declare_NegationOnly(Best_):
    """what happens if you only provide the negative? ex:  d::--init """

class Test_Declare_ParentPathNegationCommas(Best_):
    """d::Cal,Bar--Zoom/init - means pick any parents that match Cal or Bar, but filter out Zoom"""

class Test_Declare_FunctionOnlys(Best_):
    "keep only functions"

class Test_Declare_FunctionClasses(Best_):
    "keep functions and classes"

class Test_Declare_ParentNegation_AllowStandalones(Best_):
    """if you want to skip certain parents, don't skip standalone functions"""

class Test_Lines_TwoToFive(Best_):
    """display lines 2-5 """

class Test_InvalidRegex(Best_):
    "invalid regex"

class Test_Declaration_NameNegation(Best_):
    "negate matches on names"


class Test_Prepend_ReturnNotIndented(Best_):
    ...
class Test_Append_ReturnIndented(Best_):
    ...
class Test_Prepend_InvisiblesNoEffect(Best_):
    ...

class Test_NameRegex(Best_):
    ...

class Test_NameRegexNegate(Best_):
    ...

class Test_NameRegexNegateX(Best_):
    ...

class Test_Classnames(Best_):
    ...
class Test_Classnames_BadNegate(Best_):
    ...
class Test_Classnames_Negate(Best_):
    ...
class Test_Classnames_NegateX(Best_):
    ...

class Test_Inits_Parentname(Best_):
    ...

class Test_Inits_ParentnameBadNegate(Best_):
    ...

class Test_Inits_ParentnameNegate(Best_):
    ...

class Test_Inits_ParentnameNegateX(Best_):
    ...

class Test_Inits_ParentnameYesNegateX(Best_):
    ...

class Test_Classnames_CommaOrsNoPrivate(Best_):
    "use comma as `|` and no `^_`"


class Test_Declarations_FixedInsentive(Best_):
    "test that fixed insensitive works  should find `class Calculator`"

class Test_More_FixedInsentive(Best_):
    "test that fixed insensitive works  should find `class Calculator`"

class Test_Change_IOOnce(Best_):
    "change `i` to `o` once"

class Test_Change_IOAll(Best_):
    "change `i` to `o` for all"

class Test_Change_LineFeed(Best_):
    "check that \n results in line feeds"

class Test_Change_NoLineFeed(Best_):
    "check that \n doesnt result in line feeds with ::r flag included"


class Test_Shortcodes_EnvYesCommNo(Best_):
    "turn off shortcodes on command"

class Test_Shortcodes_EnvNoCommYes(Best_):
    "turn on shortcodes on command"

class Test_Shortcodes_EnvYes(Best_):
    "env has shortcodes on"

class Test_Shortcodes_EnvYesCommYes(Best_):
    "both env and command have shortcodes on"

class Test_Shortcodes_EnvNo(Best_):
    "env off"

class Test_Zsh_Declarations(Best_):
    "test zsh grammar"
    shared_lib = "zsh"

class Test_Js_Declarations(Best_):
    "test js"
    shared_lib = "js"

class Test_DeleteBlanksToo(Best_):
    "blank lines do get deleted, at least in the body of something, if followed by all"

class Test_DeleteBlanksTooStopAtDelete(Best_):
    "blank lines don't get deleted, at least in the body of something, if stopping at delete"

TU_BASE_CLASSES = (Base,Best_)

if __name__ == "__main__":


    # global breakpoints
    breakpoints = set_breakpoints3() or breakpoints

    module = f"{PA_SCRIPT.stem}"
    sys.exit(unittest.main())
