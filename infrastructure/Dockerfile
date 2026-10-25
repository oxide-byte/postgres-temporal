FROM postgres:19rc1-bookworm
ENV POSTGRES_PASSWORD password
COPY ./init.sql /docker-entrypoint-initdb.d/
