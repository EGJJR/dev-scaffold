import logging

from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware
from fastapi.middleware.trustedhost import TrustedHostMiddleware
from pydantic import BaseModel, Field

from app.config import Settings

logging.basicConfig(level=logging.INFO, format="%(levelname)s %(name)s %(message)s")
logger = logging.getLogger("{{ project_name }}")

settings = Settings()

docs_url = None if settings.is_production() else "/docs"
redoc_url = None if settings.is_production() else "/redoc"
openapi_url = None if settings.is_production() else "/openapi.json"

app = FastAPI(
    title="{{ project_name }}",
    version="0.1.0",
    docs_url=docs_url,
    redoc_url=redoc_url,
    openapi_url=openapi_url,
)

app.add_middleware(TrustedHostMiddleware, allowed_hosts=settings.host_list())
app.add_middleware(
    CORSMiddleware,
    allow_origins=settings.cors_list(),
    allow_credentials=False,
    allow_methods=["GET", "POST"],
    allow_headers=["Authorization", "Content-Type"],
)


class EchoIn(BaseModel):
    message: str = Field(min_length=1, max_length=200)


@app.get("/health")
def health() -> dict[str, str]:
    return {"status": "ok"}


@app.post("/echo")
def echo(body: EchoIn) -> dict[str, str]:
    return {"message": body.message}


logger.info("service initialized")
