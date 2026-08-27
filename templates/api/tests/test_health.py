from fastapi.testclient import TestClient


def test_health_does_not_leak_config(client: TestClient) -> None:
    response = client.get("/health")
    assert response.status_code == 200
    body = response.json()
    assert body == {"status": "ok"}
    assert "secret" not in response.text.lower()
    assert "SECRET_KEY" not in response.text


def test_echo_validates_input(client: TestClient) -> None:
    response = client.post("/echo", json={"message": ""})
    assert response.status_code == 422


def test_docs_disabled_in_non_production_test_env(client: TestClient) -> None:
    response = client.get("/docs")
    assert response.status_code == 200
