import pytest
from pydantic import ValidationError

from app.config import Settings


def test_secret_key_is_required(monkeypatch) -> None:
    monkeypatch.delenv("SECRET_KEY", raising=False)
    monkeypatch.setenv("ENV", "test")
    with pytest.raises(ValidationError):
        Settings()


def test_settings_load_when_secret_present(monkeypatch) -> None:
    monkeypatch.setenv("SECRET_KEY", "test-only-not-secret")
    monkeypatch.setenv("ENV", "test")
    settings = Settings()
    assert settings.env == "test"
