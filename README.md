# Хъ (Херъ)
Костыльный клон Твиттера/X сделанный на коленке за неделю

## Переключение между Postgres и SQLite
Чтобы собрать `api` с поддержкой SQLite достаточно сделать `cargo build`, если нужна поддержка Postgres то нужно включить фичу `postgres` - `cargo build --features postgres`

**⚠️ Собранный бинарник может поддерживать только один бэкенд БД**

### Миграция данных с SQLite на Postgres
**⚠️ Postgres и SQLite должны иметь последнию версию схемы БД перед миграцией**
```
$ POSTGRES_URL=postgres://postgres:postgres@localhost/rutwt SQLITE_URL=sqlite://main.db cargo run --bin migration-tool
```

## Конфигурация
Вся конфигурация сделана через переменные среды для удобства, вот пример конфигурации:
```
JWT_SECRET=test # Обязательно смените этот ключ на более безопасный
READ_ONLY_DATABASE_URL=postgres://postgres:postgres@localhost/rutwt # Поддерживается sqlite и postgres
READ_WRITE_DATABASE_URL=postgres://postgres:postgres@localhost/rutwt # Поддерживается sqlite и postgres
```

SQLite удобен для тестирования, в то время как Postgres рекомендован для развёртывания в продакшене из за масштабируемости

## Деплоймент
Есть файл `docker-compose.yml` для деплоймента на одну ноду с локальной репликой Postgres

Для начала надо инициализировать основную БД:
```
$ docker-compose up primary -d
$ docker-compose exec --user postgres -i primary psql
postgres=# CREATE USER replicator WITH REPLICATION ENCRYPTED PASSWORD 'replicator';
postgres=# SELECT pg_create_physical_replication_slot('replica1');
postgres=# CREATE DATABASE rutwt;
$ docker-compose exec --user postgres -T primary psql rutwt < api/data/0000-base-schema-postgres.sql
$ docker-compose exec --user postgres -T primary psql rutwt < api/data/0001-media-update-postgres.sql
```

После этого можно сделать `docker-compose up -d` для того чтобы запустить остальные сервисы