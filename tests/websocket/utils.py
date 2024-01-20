import websocket
import requests
import string
import random
backend_base_url = "localhost:3000/api"




def build_path(path: str, scheme = "http"):
    return scheme + "://" + backend_base_url + path

def debug_version():
    response = requests.get(build_path(path="/version"))
    version = response.json()
    assert version is not None, "Version is none, aborting ..."
    print(version)


def establish_websocket_connection(token: str):
    connection = websocket.create_connection(build_path("/ws?token="+token, "ws"))
    return connection


def send_and_receive_message(info: str, connection: websocket.WebSocket, send: str, expect: str):
    print(f"[{info}] --> {send}")
    connection.send(send)
    result = connection.recv()
    print(f"[{info}] <-- {result}")
    assert result == expect, info + " assertion did not work"
    print("✔ - " + info)

def receive_message(info: str, connection: websocket.WebSocket, expect: str):
    result = connection.recv()
    print(f"[{info}] <-- {result}")
    assert result == expect, info + " assertion did not work"
    print("✔ - " + info)



def login(user):
    params = {
        "username": user.get("username"),
        "password": user.get("password")
    }
    response = requests.post(build_path(path="/login"), json=params)
    response = response.json()
    return response.get("data")

def register(user, public_key):
    response = requests.post(build_path("/users"), json={
        "username": user.get("username"),
        "password": user.get("password"),
        "generate_key": False,
        "public_key": public_key
    })
    print(response)

    json = response.json()
    print(json)

def get_random_id():
    return ''.join(random.choices(string.ascii_uppercase + string.digits, k=10))


def send_friend_request(token: str, username: str):
    response = requests.post(build_path("/friend-requests"),json={ "recipient": username }, headers={"authorization": "Bearer "+token, "Content-Type": "application/json", "Accept": "application/json"})
    print(token)
    return response.json()

def accept_friend_request(token: str, uuid: str):
    response = requests.patch(build_path("/friend-requests/"+uuid), json={"accepted": True}, headers={"Authorization": "Bearer "+token})
    return response.json()