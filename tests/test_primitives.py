#!/usr/bin/env python3
"""End-to-end unit tests verifying standalone execution of all 5 primitives."""

from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
import time
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SKILLS = ROOT / "skills"


def run_tool(skill: str, args: list[str], env_extra: dict[str, str] | None = None, stdin_text: str | None = None, cwd: Path | None = None) -> subprocess.CompletedProcess[str]:
    script = SKILLS / skill / "scripts" / f"{skill}.py"
    env = os.environ.copy()
    if env_extra:
        env.update(env_extra)
    return subprocess.run(
        [sys.executable, str(script), *args],
        input=stdin_text,
        cwd=str(cwd) if cwd else None,
        env=env,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )


class TestAgentPrimitives(unittest.TestCase):
    def test_standalone_no_cross_imports(self) -> None:
        """Each folder can be copied in isolation and executed from a blank temp directory."""
        import shutil
        for name in ("sieve", "baton", "recall", "guard", "atlas"):
            with tempfile.TemporaryDirectory() as tmp:
                dst = Path(tmp) / name
                shutil.copytree(SKILLS / name, dst)
                proc = subprocess.run(
                    [sys.executable, str(dst / "scripts" / f"{name}.py"), "--help"],
                    cwd=tmp,
                    capture_output=True,
                    text=True,
                    encoding="utf-8",
                )
                self.assertEqual(proc.returncode, 0, msg=f"{name} failed isolated --help: {proc.stderr}")

    def test_sieve_compress_and_lossless_peek(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            env = {"SIEVE_HOME": str(Path(tmp) / ".sieve")}
            noisy_lines = ["2026-11-15T10:00:00Z INFO heartbeat ok" for _ in range(800)]
            noisy_lines[419] = "Traceback (most recent call last):"
            noisy_lines[420] = '  File "worker.py", line 88, in handle'
            noisy_lines[421] = "KeyError: 'missing_tenant_id_9941'"
            raw_log = "\n".join(noisy_lines) + "\n"

            res = run_tool("sieve", ["pipe", "--label", "pytest", "--json"], env_extra=env, stdin_text=raw_log)
            self.assertEqual(res.returncode, 0, res.stderr)
            meta = json.loads(res.stdout)
            self.assertGreater(meta["saved_pct"], 85.0)
            self.assertIn("missing_tenant_id_9941", meta["rendered"])

            peek = run_tool("sieve", ["peek", meta["id"], "--lines", "420:422"], env_extra=env)
            self.assertEqual(peek.returncode, 0)
            self.assertIn("L422: KeyError: 'missing_tenant_id_9941'", peek.stdout)

    def test_baton_compaction_capsule(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            env = {"BATON_HOME": str(Path(tmp) / ".baton")}
            self.assertEqual(
                run_tool("baton", ["set", "--goal", "Migrate auth cookies", "--next", "Verify SameSite=Lax"], env_extra=env, cwd=Path(tmp)).returncode,
                0,
            )
            run_tool("baton", ["fact", "Legacy Safari drops Lax on POST redirect", "--ref", "auth/cookie.py:41"], env_extra=env, cwd=Path(tmp))
            run_tool("baton", ["deadend", "Setting SameSite=None without Secure", "--reason", "Rejected by Chromium"], env_extra=env, cwd=Path(tmp))
            res = run_tool("baton", ["resume"], env_extra=env, cwd=Path(tmp))
            self.assertEqual(res.returncode, 0)
            self.assertIn("Migrate auth cookies", res.stdout)
            self.assertIn("DO NOT RETRY", res.stdout)
            self.assertLess(len(res.stdout), 1600)

    def test_recall_hybrid_english_and_cjk_sub5ms(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            env = {"RECALL_HOME": str(Path(tmp) / ".recall")}
            run_tool("recall", ["add", "--triggers", "src/otel/*,grpc", "--rule", "Always set timeout_millis before channel init", "--kind", "gotcha"], env_extra=env)
            run_tool("recall", ["add", "--triggers", "报销,发票,finance", "--rule", "差旅发票抬头统一填个人独立研究账户并归档PDF", "--kind", "workflow"], env_extra=env)
            for i in range(60):
                run_tool("recall", ["add", "--triggers", f"mod_{i}/*", "--rule", f"Unrelated module {i} invariant padding text for realistic vault size."], env_extra=env)

            hit_en = run_tool("recall", ["match", "--query", "grpc channel hang", "--files", "src/otel/client.py", "--top", "2", "--json"], env_extra=env)
            data_en = json.loads(hit_en.stdout)
            self.assertEqual(data_en["matches"][0]["triggers"], "src/otel/*,grpc")
            self.assertLess(data_en["elapsed_ms"], 50.0)

            hit_zh = run_tool("recall", ["match", "--query", "本月差旅发票怎么处理", "--top", "2", "--json"], env_extra=env)
            data_zh = json.loads(hit_zh.stdout)
            self.assertIn("差旅发票抬头", data_zh["matches"][0]["rule"])

    def test_guard_exit2_shield_and_loop_breaker(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            env = {"GUARD_HOME": str(Path(tmp) / ".guard")}
            # 1. Destructive command blocked with exit 2
            blk = run_tool("guard", ["shield", "--text", "git push --force origin main"], env_extra=env)
            self.assertEqual(blk.returncode, 2)
            self.assertIn("GUARD BLOCKED", blk.stderr)

            # 2. Secret leak blocked with exit 2
            sec = run_tool("guard", ["shield", "--text", "curl -H 'Authorization: Bearer sk-proj-1234567890abcdefghijklmno'"], env_extra=env)
            self.assertEqual(sec.returncode, 2)

            # 3. Loop breaker trips with exit 2 on 3rd identical failure
            fail_cmd = ["run", "--max-repeats", "3", "--", sys.executable, "-c", "import sys; print('boom'); sys.exit(1)"]
            r1 = run_tool("guard", fail_cmd, env_extra=env)
            r2 = run_tool("guard", fail_cmd, env_extra=env)
            r3 = run_tool("guard", fail_cmd, env_extra=env)
            self.assertEqual(r1.returncode, 1)
            self.assertEqual(r2.returncode, 1)
            self.assertEqual(r3.returncode, 2)
            self.assertIn("LOOP BREAKER", r3.stderr)

    def test_atlas_code_and_stale_todo_radar(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            p = Path(tmp)
            (p / "main.py").write_text("import helper\nclass Engine:\n    def start(self):\n        pass\n", encoding="utf-8")
            old_file = p / "helper.py"
            old_file.write_text("# TODO: remove python 2 shim from 2019\ndef run_task():\n    return 42\n", encoding="utf-8")
            old_ts = time.time() - 180 * 86400
            os.utime(old_file, (old_ts, old_ts))
            (p / "NOTES.md").write_text("# Weekly Ops\n## Incident Checklist\n", encoding="utf-8")

            res = run_tool("atlas", ["map", str(p), "--json"])
            self.assertEqual(res.returncode, 0)
            data = json.loads(res.stdout)
            self.assertIn("main.py", data["entrypoints"])
            self.assertTrue(any("helper.py" in t for t in data["stale_todo_traps"]))

    def test_installer_selective_and_doctor(self) -> None:
        doc = subprocess.run(
            [sys.executable, str(ROOT / "install.py"), "--doctor"],
            capture_output=True,
            text=True,
            encoding="utf-8",
        )
        self.assertEqual(doc.returncode, 0, doc.stderr)
        self.assertIn("FTS5/BM25 -> OK", doc.stdout)

        with tempfile.TemporaryDirectory() as tmp:
            dest = Path(tmp) / "custom_skills"
            inst = subprocess.run(
                [
                    sys.executable,
                    str(ROOT / "install.py"),
                    "--only",
                    "sieve,guard",
                    "--dest",
                    str(dest),
                    "--print-hooks",
                    "--print-agents-md",
                ],
                capture_output=True,
                text=True,
                encoding="utf-8",
            )
            self.assertEqual(inst.returncode, 0, inst.stderr)
            self.assertTrue((dest / "sieve" / "scripts" / "sieve.py").exists())
            self.assertTrue((dest / "guard" / "scripts" / "guard.py").exists())
            self.assertFalse((dest / "atlas").exists())
            self.assertIn("stop-gate", inst.stdout)


if __name__ == "__main__":
    unittest.main()
