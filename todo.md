# Todo

### Schema

Messages
Sender: varchar foreign key -> users
Recipient: varchar foreign key -> users
content: varchar()
content_sender: varchar()
...meta

Friend Requests: TODO
Sender: varchar foreign key -> users
Recipient: varchar foreign key -> users
is_accepted: bool
...meta


Users:
pubkey: varchar()

Rooms: TODO
id: primary key
name: varchar

RoomsCrossTable
room: foreign key room
user: foreign key user

### Encryption
RSA Encryption for messages

User 1 -> User 2
- message is saved with content (encrypted with User 2 pubkey) and content_sender (encrypted with User 1 pub key)

Next steps:

- p2p chat unencrypted
- p2p chat with saved messages
- p2p chat encrypted