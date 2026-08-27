from pydantic import Field
from pydantic_settings import BaseSettings, SettingsConfigDict


class Settings(BaseSettings):
    model_config = SettingsConfigDict(env_file=".env", extra="ignore")

    env: str = Field(default="development", alias="ENV")
    secret_key: str = Field(alias="SECRET_KEY")
    allowed_hosts: str = Field(default="localhost,127.0.0.1", alias="ALLOWED_HOSTS")
    cors_origins: str = Field(
        default="http://localhost:8000,http://127.0.0.1:8000",
        alias="CORS_ORIGINS",
    )

    def is_production(self) -> bool:
        return self.env.lower() == "production"

    def host_list(self) -> list[str]:
        if self.is_production():
            hosts = [
                item.strip()
                for item in self.allowed_hosts.split(",")
                if item.strip() and item.strip() != "*"
            ]
            return hosts or ["localhost", "127.0.0.1"]
        return ["*"]

    def cors_list(self) -> list[str]:
        origins = [
            item.strip()
            for item in self.cors_origins.split(",")
            if item.strip()
        ]
        if self.is_production():
            return [origin for origin in origins if origin != "*"]
        return origins
