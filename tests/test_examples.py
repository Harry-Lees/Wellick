import os
import platform
import subprocess
from os import PathLike
from pathlib import Path

import pytest


def run_example(file: PathLike) -> tuple[bytes, bytes, int]:
    """
    Run an example and check that it compiles.

    Returns a tuple of stdout, stderr, and the return code.
    """

    compiler_output = subprocess.run(["cargo", "run", "--release", str(file)])
    if compiler_output.returncode != 0:
        raise RuntimeError("Failed to compile")

    if platform.system() == "Windows":
        result = subprocess.run(["LINK", "a.out", "builtins", "/MD", "/Wall"], shell=True)
        result = subprocess.run(["a.exe"], shell=True)
        return result.stdout, result.stderr, result.returncode

    raise NotImplementedError(
        f"Unsuported platform, expected Windows, found {platform.system()}"
    )


@pytest.mark.parametrize("file", [
    "fibonacci.wellick",
    "pointer_qualifier.wellick",
    "iadd_builtin.wellick",
    "isub_builtin.wellick",
    "imul_builtin.wellick",
    "ptr_type.wellick",
])
def test_pass_examples(file: str) -> None:
    """Test that the builtins example compiles."""
    root = Path(__file__).parent
    file_path = root / "pass_examples" / file

    stdout, stderr, retcode = run_example(file_path)
    assert retcode == 0


@pytest.mark.parametrize("file", ["default_const.wellick"])
def test_fail_examples(file: str) -> None:
    """Test that the default_const example compiles."""
    root = Path(__file__).parent
    file_path = root / "fail_examples" / file

    with pytest.raises(RuntimeError):
        stdout, stderr, retcode = run_example(file_path)
