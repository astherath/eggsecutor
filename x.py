#!/usr/bin/env python
import os
from pathlib import Path
import json
from datetime import datetime
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
def generate_cov():
    """generates cov file(s)"""
    # check flag is set before starting
    os.system('export RUSTFLAGS="-Zinstrument-coverage"')
    cmd = " ".join([
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

    append_cov_data_to_file()


@cli.command()
def test():
    """runs tests and generates cov file(s)"""
    # check flag is set before starting
    os.system('export RUSTFLAGS="-Zinstrument-coverage"')
    test_cmd = " ".join(["cargo", "test"])
    os.system(test_cmd)
    cmd = " ".join([
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

    append_cov_data_to_file()


def get_cov_data_path() -> str:
    return "coverage_data.csv"


def append_cov_data_to_file():
    cov_data_path = get_cov_data_path()
    if not os.path.exists(cov_data_path):
        create_header_for_file()
    data = get_output_ready_data()
    with open(cov_data_path, "a") as f:
        f.write(data)


def get_output_ready_data() -> str:
    timestamp = datetime.now()
    cov_percent = get_cov_percentage()
    sloc = get_current_sloc()
    return f"{timestamp},{cov_percent},{sloc}\n"


def get_current_sloc() -> str:
    sloc_str = os.popen("sloc src/ tests/").read().replace("\n", "").replace(
        " ", "")
    start = sloc_str.index(":") + 1
    end = start + sloc_str[start:].index("Source")
    sloc = sloc_str[start:end]
    return sloc


def create_header_for_file():
    header_str = "datetime,cov(%),sloc\n"
    cov_data_path = get_cov_data_path()
    with open(cov_data_path, "w") as f:
        f.write(header_str)


def get_cov_percentage() -> str:
    generated_cov_path = str(Path("./coverage/coverage.json"))
    with open(generated_cov_path, "r") as f:
        data = json.loads(f.read())
    return data["message"]


def create_cov_file_if_not_exists():
    pass


if __name__ == "__main__":
    cli()
