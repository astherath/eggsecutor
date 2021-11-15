#!/usr/bin/env python
import os
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
def test():
    """runs tests and generates cov file(s)"""
    cmd = ";".join([
        'RUSTFLAGS="-Z instrument-coverage"',
        'LLVM_PROFILE_FILE="json5format-%m.profraw"', 'cargo test'
    ])
    os.system(cmd)


if __name__ == "__main__":
    cli()
