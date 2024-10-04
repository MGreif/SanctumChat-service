# Todo

### Schema

Rooms: TODO
id: primary key
name: varchar

RoomsCrossTable
room: foreign key room
user: foreign key user

### Encryption
[] Maybe introduce a better fitting encryption algorithm as RSA is quite slow.


### Security
[] Implement a brute-force protection
[] Include client details (user-agent, ...) into token to ensure a more secure session handling and prevent simple forms of session hijacking
[] Perform fuzzing of endpoints to prevent errors based on content type or length
[] Prevent any error leakage from infrastructure (database errors)
[] Analyse login flow and taken time. The time taken could be used to enumerate registered users. Prevent this


### Usability
[] Improve session handling as its currently in memory
[] Add logic to programatically apply migrations based on env var
[] Implement a proper boundary between infrastructure and domain. E.g. domain functions should not return HTTP errors.

### Database
[] Introduce indices into postgreSQL
[] Implement a backup strategy
