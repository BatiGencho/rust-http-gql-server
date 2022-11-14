# syntax=docker/dockerfile:1.3-labs
FROM rust:slim-buster

RUN apt-get update -qq && \
    apt-get install -y libpq-dev libsqlite3-dev default-libmysqlclient-dev wget && \
    rm -rf /var/lib/apt/lists/* && \
    \
    cargo install diesel_cli --features postgres

RUN wget https://raw.githubusercontent.com/vishnubob/wait-for-it/master/wait-for-it.sh && \
    chmod +x wait-for-it.sh

COPY ./migrations /migrations
COPY ./entrypoint.sh /entrypoint.sh
RUN chmod +x ./entrypoint.sh

#ENTRYPOINT /bin/bash -c "./wait-for-it.sh postgres:5432 --strict --timeout=30 -- /entrypoint.sh"
ENTRYPOINT ["./entrypoint.sh"]
