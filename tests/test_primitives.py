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
        for name in ("sieve", "baton", "recall", "guard", "atlas", "probe", "splice"):
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

    def test_probe_slice_run_and_diff_microscope(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            p = Path(tmp)
            pkg = p / "demo"
            pkg.mkdir()
            (pkg / "__init__.py").write_text("from .core import Engine, Packet\n", encoding="utf-8")
            (pkg / "dead_unused.py").write_text("class UnusedDistractor:\n    def nope(self) -> int:\n        return 0\n", encoding="utf-8")
            (pkg / "store.py").write_text(
                "class DedupStore:\n"
                "    def __init__(self) -> None:\n"
                "        self._seen = {}\n"
                "    def record(self, digest: str) -> None:\n"
                "        self._seen[digest] = 1\n"
                "    def rollback(self, token: str) -> None:\n"
                "        self._seen.pop(token, None)\n",
                encoding="utf-8",
            )
            (pkg / "core.py").write_text(
                "from .store import DedupStore\n"
                "class Packet:\n"
                "    def __init__(self, pid: str, payload: dict) -> None:\n"
                "        self.pid = pid\n"
                "        self.payload = payload\n"
                "    def clone_shallow(self) -> 'Packet':\n"
                "        return Packet(self.pid, dict(self.payload))\n"
                "def mutate_price(pkt: Packet) -> Packet:\n"
                "    pkt.payload['nested']['amt'] *= 100\n"
                "    return pkt\n"
                "class Engine:\n"
                "    def __init__(self) -> None:\n"
                "        self.store = DedupStore()\n"
                "    def run_once(self, pkt: Packet) -> Packet:\n"
                "        snap = pkt.clone_shallow()\n"
                "        mutate_price(pkt)\n"
                "        self.store.record('sha256:abcd')\n"
                "        self.store.rollback(pkt.pid)\n"
                "        return snap\n",
                encoding="utf-8",
            )

            # 1. Reachability slice prunes dead_unused.py
            sl = run_tool("probe", ["slice", str(p), "--entry", "demo", "--json"])
            self.assertEqual(sl.returncode, 0, sl.stderr)
            sl_data = json.loads(sl.stdout)
            self.assertEqual(sl_data["pruned_unreferenced_count"], 1)
            self.assertIn("demo/dead_unused.py", sl_data["pruned_files_sample"])

            # 2. Causal runtime trace detects ALIAS HAZARD, IN-PLACE MUTATION, and STATE NO-OP
            tr = run_tool(
                "probe",
                [
                    "run",
                    "--cwd",
                    str(p),
                    "--focus",
                    "demo",
                    "--code",
                    "from demo import Engine, Packet; Engine().run_once(Packet('id-1', {'nested': {'amt': 5}}))",
                    "--json",
                ],
            )
            self.assertEqual(tr.returncode, 0, tr.stderr)
            tr_data = json.loads(tr.stdout)
            self.assertGreaterEqual(len(tr_data["alias_hazards"]), 1)
            self.assertGreaterEqual(len(tr_data["mutation_alerts"]), 1)
            self.assertGreaterEqual(len(tr_data["state_noop_alerts"]), 1)

            # 3. Differential probe catches bool-vs-int cache collision & caller input mutation
            (pkg / "buggy_eval.py").write_text(
                "class CachedCheck:\n"
                "    def __init__(self, use_cache: bool) -> None:\n"
                "        self.use_cache = use_cache\n"
                "        self.cache = {}\n"
                "    def eval(self, env: dict) -> bool:\n"
                "        key = tuple(sorted(env.items()))\n"
                "        if self.use_cache and key in self.cache:\n"
                "            return self.cache[key]\n"
                "        env['_touched'] = True\n"
                "        res = (env.get('flag') is True)\n"
                "        if self.use_cache:\n"
                "            self.cache[key] = res\n"
                "        return res\n",
                encoding="utf-8",
            )
            df = run_tool(
                "probe",
                [
                    "diff",
                    "--cwd",
                    str(p),
                    "--setup",
                    "from demo.buggy_eval import CachedCheck; a=CachedCheck(True); b=CachedCheck(False)",
                    "--fn-a",
                    "lambda d: a.eval(d)",
                    "--fn-b",
                    "lambda d: b.eval(d)",
                    "--vars",
                    "flag",
                    "--json",
                ],
            )
            self.assertEqual(df.returncode, 1)  # Exits 1 when divergence/drift/mutation found
            df_data = json.loads(df.stdout)
            self.assertFalse(df_data["equivalent"])
            self.assertGreaterEqual(len(df_data["stateful_cache_drifts"]), 1)
            self.assertGreaterEqual(len(df_data["caller_input_mutations"]), 1)

    def test_splice_ast_view_crlf_atomic_patch_and_rollback(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            p = Path(tmp)
            pkg = p / "svc"
            pkg.mkdir()
            (pkg / "__init__.py").write_text("", encoding="utf-8")
            # Write a CRLF + UTF-8 (CJK + em-dash) Python file
            orig_py = (
                "# 核心服务 — CRLF Unicode header\r\n"
                "class Worker:\r\n"
                "    def compute(self, x: int) -> int:\r\n"
                "        return x + 1   \r\n"
                "    def untouched(self) -> str:\r\n"
                "        return 'ok'\r\n"
            ).encode("utf-8")
            f_py = pkg / "worker.py"
            f_py.write_bytes(orig_py)
            (p / "GUIDE.md").write_text("# Top\n## Deploy Steps\nRun migrate first.\n## Rollback\nRestore snapshot.\n", encoding="utf-8")

            # 1. AST view extracts only Worker.compute
            vw = run_tool("splice", ["view", str(f_py), "--symbol", "Worker.compute"])
            self.assertEqual(vw.returncode, 0, vw.stderr)
            self.assertIn("def compute", vw.stdout)
            self.assertNotIn("def untouched", vw.stdout)

            # 2. Markdown heading view
            vmd = run_tool("splice", ["view", str(p / "GUIDE.md"), "--heading", "Deploy Steps"])
            self.assertEqual(vmd.returncode, 0)
            self.assertIn("Run migrate first.", vmd.stdout)
            self.assertNotIn("Restore snapshot.", vmd.stdout)

            # 3. Atomic rollback when 2nd op introduces SyntaxError
            bad_tx = json.dumps([
                {"path": "svc/worker.py", "op": "replace", "old": "return x + 1", "new": "return x * 10"},
                {"path": "svc/broken.py", "op": "write", "content": "def oops(:\n    pass\n"},
            ])
            rb = run_tool("splice", ["apply", "--cwd", str(p), "--stdin"], stdin_text=bad_tx)
            self.assertEqual(rb.returncode, 1)
            self.assertEqual(f_py.read_bytes(), orig_py)
            self.assertFalse((pkg / "broken.py").exists())

            # 4. Valid replace_symbol preserves CRLF & UTF-8 header
            good_tx = json.dumps([
                {
                    "path": "svc/worker.py",
                    "op": "replace_symbol",
                    "symbol": "Worker.compute",
                    "new": "def compute(self, x: int) -> int:\n    return x * 100",
                }
            ])
            ap = run_tool("splice", ["apply", "--cwd", str(p), "--stdin"], stdin_text=good_tx)
            self.assertEqual(ap.returncode, 0, ap.stderr)
            after_b = f_py.read_bytes()
            self.assertIn(b"\r\n", after_b)
            self.assertIn("核心服务 —".encode("utf-8"), after_b)
            self.assertIn(b"return x * 100", after_b)

            chk = run_tool("splice", ["check", str(p)])
            self.assertEqual(chk.returncode, 0, chk.stderr)

    def test_audit_sessions_local_telemetry(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            fake_home = Path(tmp)
            agy_log = fake_home / ".gemini" / "antigravity-cli" / "brain" / "conv1" / ".system_generated" / "logs" / "transcript.jsonl"
            agy_log.parent.mkdir(parents=True, exist_ok=True)
            steps = [
                {"type": "PLANNER_RESPONSE", "tool_calls": [{"name": "read_file", "args": {"Path": "src/a.py"}}]},
                {"type": "PLANNER_RESPONSE", "tool_calls": [{"name": "read_file", "args": {"Path": "src/a.py"}}]},
                {"type": "PLANNER_RESPONSE", "tool_calls": [{"name": "run_command", "args": {"CommandLine": "python -c \"import foo\""}}]},
                {"type": "ERROR_MESSAGE", "content": "ModuleNotFoundError: No module named 'foo'\nThe tool's output was truncated due to being too long."},
            ]
            agy_log.write_text("\n".join(json.dumps(s) for s in steps) + "\n", encoding="utf-8")

            grok_log = fake_home / ".grok" / "sessions" / "proj1" / "sid1" / "updates.jsonl"
            grok_log.parent.mkdir(parents=True, exist_ok=True)
            grok_events = [
                {"params": {"update": {"sessionUpdate": "tool_call", "title": "read_file", "rawInput": {"file_path": "pkg/x.py"}}}},
                {"params": {"update": {"sessionUpdate": "tool_call", "title": "read_file", "rawInput": {"file_path": "pkg/x.py"}}}},
                {"params": {"update": {"sessionUpdate": "tool_call_update", "status": "completed", "rawOutput": "x" * 9000}}},
            ]
            grok_log.write_text("\n".join(json.dumps(e) for e in grok_events) + "\n", encoding="utf-8")

            proc = subprocess.run(
                [sys.executable, str(ROOT / "audit" / "audit_sessions.py"), "--home", str(fake_home), "--json"],
                capture_output=True,
                text=True,
                encoding="utf-8",
            )
            self.assertEqual(proc.returncode, 0, proc.stderr)
            rep = json.loads(proc.stdout)
            self.assertIn("antigravity_cli", rep["harnesses_detected"])
            self.assertIn("grok_cli", rep["harnesses_detected"])
            rec_names = [r["primitive"] for r in rep["recommended_primitives"]]
            self.assertIn("probe", rec_names)
            self.assertIn("splice", rec_names)
            self.assertIn("sieve", rec_names)

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
