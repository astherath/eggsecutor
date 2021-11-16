#!/usr/bin/env python
import os
from pathlib import Path
import click


@click.group()
def cli():
    pass


@cli.command()
def clean():
    """cleans cov files"""
    files = [x for x in os.listdir() if x.endswith("profraw")]
    for file_path in files:
        os.remove(file_path)


@cli.command()
def view():
    """views the geenrated html files in default browser"""
    os.system(" ".join(["open", str(Path("coverage/src/index.html"))]))


@cli.command()
def test():
    """runs tests and generates cov file(s)"""
    cmd = ";".join([
        "grcov",
        ".",
        "--binary-path",
        "./target/debug",
        "-s",
        ".",
        "-t",
        "html",
        "--branch",
        "--ignore-not-existing",
        "-o",
        "./coverage/",
    ])
    os.system(cmd)


if __name__ == "__main__":
    cli()
