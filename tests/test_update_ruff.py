import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import update_ruff


class UpdateRuffTests(unittest.TestCase):
    def test_parser_update_does_not_change_runtime_requirement_or_package_version(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            cargo = root / "Cargo.toml"
            project = root / "pyproject.toml"
            readme = root / "README.md"
            cargo.write_text(
                '[package]\nversion = "0.27.0"\n'
                '[dependencies]\n'
                'ruff_python_parser = { git = "https://github.com/astral-sh/ruff", tag = "0.17.2" }\n'
                'other = { git = "https://example.com/other", tag = "0.17.2" }\n'
            )
            project.write_text('[project]\ndependencies = ["ruff>=0.11.0"]\n')
            readme.write_text("Ruff minimum 0.11.0\n")

            with patch.object(update_ruff, "ROOT", root), patch.object(
                update_ruff, "latest_ruff_tag", return_value="0.17.3"
            ):
                update_ruff.main()

            self.assertIn('version = "0.27.0"', cargo.read_text())
            self.assertIn('astral-sh/ruff", tag = "0.17.3"', cargo.read_text())
            self.assertIn('example.com/other", tag = "0.17.2"', cargo.read_text())
            self.assertEqual(project.read_text(), '[project]\ndependencies = ["ruff>=0.11.0"]\n')
            self.assertEqual(readme.read_text(), "Ruff minimum 0.11.0\n")

    def test_current_parser_version_is_unchanged(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            cargo = root / "Cargo.toml"
            original = 'parser = { git = "https://github.com/astral-sh/ruff", tag = "0.17.2" }\n'
            cargo.write_text(original)
            with patch.object(update_ruff, "ROOT", root), patch.object(
                update_ruff, "latest_ruff_tag", return_value="0.17.2"
            ):
                update_ruff.main()
            self.assertEqual(cargo.read_text(), original)


if __name__ == "__main__":
    unittest.main()
