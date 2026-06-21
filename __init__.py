# Hermes runtime plugin bootstrap for bl1nk-doc-mcp
from __future__ import annotations

import shutil
import subprocess
import sys
from pathlib import Path


def _find_repo_root(ctx) -> str:
    """Resolve repo root from plugin config or ctx, falling back to cwd."""
    repo_root = (
        getattr(ctx, "config", {}).get("repo_root")
        or getattr(ctx, "repo_root", None)
    )
    if repo_root:
        return str(repo_root)

    return str(Path.cwd())


def _resolve_binary() -> str | None:
    """Return path to bl1nk-doc-mcp binary if it exists on PATH or nearby."""
    binary = shutil.which("bl1nk-doc-mcp")
    if binary:
        return binary

    # Fallback: check relative to this file (repo_root/target/release/...)
    candidate = Path(__file__).resolve().parent.parent / "target" / "release" / "bl1nk-doc-mcp"
    if candidate.exists():
        return str(candidate)

    return None


def register(ctx) -> None:
    """Register the bl1nk-doc-mcp plugin with the Hermes runtime.

    Called by the Hermes plugin manager when the plugin is loaded.
    """
    repo_root = _find_repo_root(ctx)
    binary = _resolve_binary()

    ctx.log(f"[bl1nk-doc-mcp] registering plugin (repo_root={repo_root}, binary={binary})")

    if binary is None:
        ctx.log(
            "[bl1nk-doc-mcp] WARNING: bl1nk-doc-mcp binary not found. "
            "Run `cargo build --release` to produce the backend. "
            "Plugin registered but MCP tools will not be available until the binary exists."
        )
        return

    # Expose plugin metadata + launch config for the runtime.
    ctx.register_backend(
        name="bl1nk-doc-mcp",
        kind="mcp",
        command=[binary, repo_root],
        env={
            "BL1NK_REPO_ROOT": repo_root,
        },
        capabilities=["tools", "resources", "prompts"],
    )

    ctx.log("[bl1nk-doc-mcp] backend registered successfully")
