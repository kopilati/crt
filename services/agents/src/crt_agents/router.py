"""Generic agent endpoint using OpenAI's Responses streaming API."""

from __future__ import annotations

import asyncio
import json
import logging
import os
from functools import lru_cache
from pathlib import Path
from typing import Dict, Optional, Tuple

import yaml
from fastapi import APIRouter, HTTPException
from openai import OpenAI
from pydantic import BaseModel, Field

logger = logging.getLogger(__name__)

DEFAULT_MODEL = os.getenv("OPENAI_MODEL", "gpt-4o-mini")
CONFIG_DIR = Path(__file__).resolve().parent / "config"
STUB_ENABLED = os.getenv("STUB", "").strip().lower() in {"1", "true", "yes", "on"}
STUB_DIR = Path(os.getenv("AGENT_STUB_DIR", CONFIG_DIR))

router = APIRouter(prefix="/agents", tags=["agents"])
_client: Optional[OpenAI] = None


def _get_client() -> OpenAI:
    global _client
    if _client is None:
        _client = OpenAI()
    return _client


class AgentRunRequest(BaseModel):
    user_id: Optional[str] = Field(default=None, description="Optional user/session id")
    message: str = Field(..., description="User message/input")


class AgentRunResponse(BaseModel):
    output_text: str
    run_id: str


def _load_agent_config(agent_name: str) -> Dict[str, str]:
    for ext in ("yml", "yaml"):
        candidate = CONFIG_DIR / f"{agent_name}.{ext}"
        if candidate.exists():
            break
    else:  # pragma: no cover
        raise FileNotFoundError(f"No agent definition found for '{agent_name}'")

    with candidate.open("r", encoding="utf-8") as handle:
        data = yaml.safe_load(handle) or {}

    if "instructions" not in data:
        raise ValueError(f"Agent definition '{candidate.name}' is missing 'instructions'")

    return {
        "name": data.get("name", agent_name),
        "model": data.get("model", DEFAULT_MODEL),
        "instructions": data["instructions"],
    }


@lru_cache(maxsize=32)
def _agent_config(agent_name: str) -> Dict[str, str]:
    return _load_agent_config(agent_name)


def _extract_text(response: object) -> str:
    text = getattr(response, "output_text", None)
    if text:
        return text
    output = getattr(response, "output", None) or []
    if output:
        first = output[0]
        content = getattr(first, "content", None) or []
        if content:
            text_block = content[0]
            return getattr(text_block, "text", "")
    return ""


def _maybe_stub_response(agent_name: str) -> Optional[Tuple[str, str]]:
    if not STUB_ENABLED:
        return None

    candidate = STUB_DIR / f"{agent_name}.json"
    if not candidate.exists():
        return None

    try:
        raw = candidate.read_text(encoding="utf-8")
    except OSError as exc:  # pragma: no cover - filesystem errors
        logger.error("Failed to read stub file %s: %s", candidate, exc)
        raise HTTPException(status_code=500, detail="Stub response unavailable") from exc

    try:
        payload = json.loads(raw)
    except json.JSONDecodeError as exc:
        logger.error("Stub file %s contains invalid JSON: %s", candidate, exc)
        raise HTTPException(status_code=500, detail="Stub response invalid") from exc

    run_id = f"stub-{agent_name}"
    if isinstance(payload, dict):
        if "run_id" in payload and payload["run_id"]:
            run_id = str(payload["run_id"])
        else:
            payload["run_id"] = run_id
        output_text = json.dumps(payload, ensure_ascii=False)
    else:
        output_text = raw

    logger.info("Serving stubbed agent response for '%s' using %s", agent_name, candidate)
    return output_text, run_id


def _run_agent_sync(config: Dict[str, str], message: str) -> Tuple[str, str]:
    client = _get_client()
    instructions = config["instructions"]
    prompt = f"{instructions}\n\nUser:\n{message}"

    with client.responses.stream(model=config["model"], input=prompt) as stream:
        initial_response = getattr(stream, "response", None)
        run_id = ""
        if initial_response is not None:
            run_id = getattr(initial_response, "id", "") or ""
        for event in stream:
            event_type = getattr(event, "type", getattr(event, "event", type(event).__name__))
            if event_type == "response.output_text.delta":
                delta = getattr(event, "delta", None)
                if delta:
                    logger.info("agent delta: %s", delta)
            else:
                logger.info("agent event: %s", event_type)
        final = stream.get_final_response()
        if not run_id:
            run_id = getattr(final, "id", "") or run_id

    output_text = _extract_text(final)
    if not output_text:
        output_text = "(no text output)"
    logger.debug("result %s", output_text)
    return output_text, run_id


@router.post("/{agent_name}/run", response_model=AgentRunResponse)
async def run_agent(agent_name: str, request: AgentRunRequest) -> AgentRunResponse:
    stubbed = _maybe_stub_response(agent_name)
    if stubbed is not None:
        output_text, run_id = stubbed
        return AgentRunResponse(output_text=output_text, run_id=run_id)

    try:
        config = _agent_config(agent_name)
    except FileNotFoundError as exc:
        raise HTTPException(status_code=404, detail=str(exc)) from exc
    except ValueError as exc:
        raise HTTPException(status_code=500, detail=str(exc)) from exc

    try:
        output_text, run_id = await asyncio.to_thread(
            _run_agent_sync, config, request.message
        )
    except Exception as exc:  # pragma: no cover
        raise HTTPException(status_code=500, detail=str(exc)) from exc

    return AgentRunResponse(output_text=output_text, run_id=run_id)
