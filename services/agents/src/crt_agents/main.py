import os

import uvicorn

from .app import create_app


def run() -> None:
    host = os.getenv("BACKEND_HOST", "0.0.0.0")
    port = int(os.getenv("BACKEND_PORT", "3000"))
    uvicorn.run(create_app(), host=host, port=port)


if __name__ == "__main__":
    run()
