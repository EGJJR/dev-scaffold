import logging
import signal
import threading

from app.config import Settings

logging.basicConfig(level=logging.INFO, format="%(levelname)s %(name)s %(message)s")
logger = logging.getLogger("{{ project_name }}")


def run() -> None:
    settings = Settings()
    stop = threading.Event()

    def handle_signal(signum: int, _frame: object) -> None:
        logger.info("shutdown requested", extra={"signal": signum})
        stop.set()

    signal.signal(signal.SIGTERM, handle_signal)
    signal.signal(signal.SIGINT, handle_signal)

    logger.info("worker started")
    while not stop.is_set():
        logger.info("heartbeat")
        stop.wait(timeout=settings.poll_interval_seconds)
    logger.info("worker stopped")


def main() -> None:
    run()


if __name__ == "__main__":
    main()
