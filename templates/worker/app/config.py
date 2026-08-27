from pydantic import Field
from pydantic_settings import BaseSettings, SettingsConfigDict


class Settings(BaseSettings):
    model_config = SettingsConfigDict(env_file=".env", extra="ignore")

    env: str = Field(default="development", alias="ENV")
    secret_key: str = Field(alias="SECRET_KEY")
    poll_interval_seconds: float = Field(default=5.0, alias="POLL_INTERVAL_SECONDS")

    def is_production(self) -> bool:
        return self.env.lower() == "production"
