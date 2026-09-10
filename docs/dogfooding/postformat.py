#!/usr/bin/env python
""" perform some "complicated post processing on the raw md candidate files.
- enhances table formatting coming out of rxedit --help to follow markdown table standard
- {{root}} and {{commands}} links are computed relative to the current file's location.
"""
from pathlib import Path
import re
import os

#################################################################
# Dependencies
#################################################################
import click

def cpdb(*args, **kwargs):
    "allows conditional breakpoints on exceptions"
    return os.getenv("cpdb")

@click.command()
@click.argument('fnp_md', type=click.Path(exists=True, dir_okay=False))
@click.argument('fnp_out', type=click.Path(dir_okay=False))
def run(**kwargs):
    """delegates the work to Main"""
    try:
        mgr = Main(**kwargs)
        res = mgr.process()
        return res
    # pragma: no cover pylint: disable=unused-variable
    except (Exception,) as e:
        if cpdb():
            breakpoint()
        raise


class WaitingState:
    """a basic state machine finds out when a table representation like below starts and ends

    this done by testin for the unusual separation line after the Column names.

    Name                    Format                 Example          Description
    ----------------------  ---------------------  ---------------  -----------
    after                   A(\d+)                 ::A16            Show NUM lines after each match.
    before                  B(\d+)                 ::B5             Show NUM lines before each match.

    WaitingState flips its own class to ActiveState which will format the table to markdown.
    ActiveState in turn watches when the line passed is in not in the above format anymore
    and flips to WaitingState.  ActiveState delegates the formatting and testing to its Gapper instance.

    """

    def _ante(self,  ix : int, line : str, lines : list[str]):
        pass
    def _post(self,  ix : int, line : str, lines : list[str]):
        pass

    patre2 = re.compile("--+")

    def __init__(self, main : "Main", linecount:int):
        self.main = main
        self.linecount = linecount

    def feed(self, ix : int, line : str, lines : list[str]):
        "drives the state machine"
        self._ante(ix,line,lines)
        res = self._feed(ix,line,lines)
        self._post(ix,line,lines)
        return res

    def _feed(self, ix: int, line : str, lines : list[str]):
        """the thing here is that we are looking for `--- --- ---` on the next line,
        as a header line just has column names"""
        try:
            if ix < self.linecount - 1:
                linenext = lines[ix+1]

                li_match = [v for v in self.patre2.finditer(linenext)]
                if len(li_match) > 1:
                    self.__class__ = ActiveState
                    self.initialize_state(ix,line,lines, li_match)

            return self
        #pragma: no cover pylint: disable=unused-variable
        except (Exception,) as e:
            if cpdb():
                breakpoint()
            raise


class Gapper:
    """responsible for tracking and formatting the exact incoming `--- --- ---` lines"""
    def __init__(self, spans: list[tuple[int,int]]) -> None:
        self.spans = spans
        self.gapslices : list[tuple[int,int]] = []
        lastspan = None
        for span in spans:
            if lastspan:
                gapslice = (lastspan[1] , span[0]+1)
                self.gapslices.append(gapslice)
            lastspan = span
        self.bounds = [span[0-1] for span in self.spans[:-1]]

    def check_active(self, line : str) -> bool:
        "does this line look like the columns we started with?"
        active = len(line) > self.gapslices[-1][1]
        if active:
            for gapslice in self.gapslices:
                splice = line[gapslice[0]:gapslice[1]]
                active = (splice[:-1].strip() == "")
                if not active:
                    break
        return active

    def reformat(self, line : str) -> str:
        "make a given line into a markdown table row"
        line = line.replace(r"|",r"\|")
        line2 = "".join([c2 for ix, c in enumerate(line) if (c2:=(c if not ix in self.bounds else '|'))])
        return f"|{line2}|"

class ActiveState(WaitingState):
    """check/process if we are still in what should be a markdown table"""

    def initialize_state(self, ix : int, line : str, lines : list[str], li_match : list[re.Match]):
        """keep track for where the columns are, mostly"""
        try:
            self.li_span = [v.span() for v in li_match]
            self.gapper = Gapper(self.li_span)
            self.gapslices = []
            lastspan = None
            for ix, span in enumerate(self.li_span):
                if lastspan:
                    gapslice = (lastspan[1] , span[0]+1)
                    self.gapslices.append(gapslice)
                lastspan = span
            self.ix = ix
        #pragma: no cover pylint: disable=unused-variable
        except (Exception,) as e:
            if cpdb():
                breakpoint()
            raise

    def _post(self,  ix : int, line : str, lines : list[str]):
        "format the line as a markdown table row"
        try:
            self.main.lines[ix] = self.gapper.reformat(line)
        #pragma: no cover pylint: disable=unused-variable
        except (Exception,) as e:
            if cpdb():
                breakpoint()
            raise

    def _feed(self, ix : int, line : str, lines : list[str]):
        "transition out of the markdown table formatting state"
        try:
            active = self.gapper.check_active(line)
            if not active:
                self.__class__ = WaitingState
        #pragma: no cover pylint: disable=unused-variable
        except (Exception,) as e:
            if cpdb():
                breakpoint()
            raise


class Main:
    """ manages one run on an input and output file pair """

    def __init__(self, fnp_md : Path|str, fnp_out: Path|str, **kwargs): # pylint: disable=missing-function-docstring
        """
        fnp_md holds the raw markdown, as already process rxedit replacements for links.
        fnp_out points to $diroiroot/docs/src
        `self.state` is a basic state machine tracking if a line in the raw help has started a table representation
        for example, the flags
        """
        self.pa_in = Path(fnp_md)
        self.pa_out = Path(fnp_out)
        self.lines = self.pa_in.read_text().splitlines()
        self.state = WaitingState(self, len(self.lines))
        self.__dict__.update(**kwargs)

        # need to link other pages/markdowns relative to current file location
        if Path(fnp_out).parent.name == "commands":
            #we're a command
            self.relative_root_path = ".."
            self.relative_commands_path ="."
        else:
            self.relative_root_path = "."
            self.relative_commands_path ="./commands"



    def compute_relative_page_links(self, line : str) -> str:
        line = line.replace("{{root}}", self.relative_root_path)
        line = line.replace("{{commands}}", self.relative_commands_path)
        return line


    def process(self):
        "transform the incoming file "
        try:
            for ix, line in enumerate(self.lines):
                self.state.feed(ix, line, self.lines)

            self.lines = [self.compute_relative_page_links(line) for line in self.lines]

            with self.pa_out.open("w") as fo:
                for line in self.lines:
                    fo.write(line)
                    fo.write("\n")
        # pragma: no cover pylint: disable=unused-variable
        except (Exception,) as e:
            if cpdb(): breakpoint()
            raise

if __name__ == "__main__":
    run()
