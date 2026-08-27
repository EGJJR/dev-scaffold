from app.config import Settings


def test_production_rejects_wildcard_cors(monkeypatch) -> None:
    monkeypatch.setenv("SECRET_KEY", "test-only-not-secret")
    monkeypatch.setenv("ENV", "production")
    monkeypatch.setenv("CORS_ORIGINS", "*")
    monkeypatch.setenv("ALLOWED_HOSTS", "*")
    settings = Settings()
    assert settings.is_production()
    assert "*" not in settings.cors_list()
    assert "*" not in settings.host_list()
