## This project was created solely for fulfilling a technical assignment and is not intended for any other use. Use at your own risk.


### Способ проверки через curl

Запуск сервера

```bash
cargo run
```

Создаем топик

```bash
curl -X POST -H "Content-Type: application/json" \
-d '{"name": "my_topic", "retention": 3600, "compaction": false}' \
http://localhost:8080/create_topic
```
 
можем подписаться на топик и ждем сообщения

```
curl -N http://localhost:8080/subscribe?topic=my_topic
```

обратное действие

```bash
curl -X POST \
     -H "Content-Type: application/json" \
     -H "X-Client-Id: ваш_client_id" \
     -d '{"topic": "my_topic"}' \
     http://localhost:8080/unsubscribe
```

создаем собщение 


- topic: название топика.
- key: опциональный ключ сообщения. Если не нужно, укажите null.
- payload: содержимое сообщения.
- require_ack: требуется ли подтверждение доставки (true или false).

```bash
    curl -X POST -H "Content-Type: application/json" \
    -d '{
        "topic": "my_topic",
        "key": null,
        "payload": ":D",
        "require_ack": false
    }' \
    http://localhost:8080/publish
```
ещё несколько вариантов

```bash
curl -X POST -H "Content-Type: application/json" \
-d '{"topic":"my_topic", "key":":P", "payload":"Первое сообщение", "require_ack":false}' \
http://localhost:8080/publish


# тут уже есть require_ack
curl -X POST -H "Content-Type: application/json" \
-d '{"topic":"my_topic", "key":":O", "payload":"Второе сообщение", "require_ack":true}' \
http://localhost:8080/publish


curl -X POST -H "Content-Type: application/json" \
-d '{
    "topic": "my_topic",
    "client_id": "<ВАШ_CLIENT_ID>",
    "message_id": "<MESSAGE_ID>"
}' \
http://localhost:8080/ack

```
