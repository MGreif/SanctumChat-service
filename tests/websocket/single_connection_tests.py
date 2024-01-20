import websocket
import utils
import constants

def perform_single_connection_tests(user):
    token = utils.login(user)
    assert token is not None, "Token is none, aborting ..."
    connection = utils.establish_websocket_connection(token=token)
    connection.recv() #online users

    utils.send_and_receive_message("deserialization",
                             connection,
                             constants.deserialize,
                             constants.expect_deserialize
                             )
    
    print("Message deserialization works")

    utils.send_and_receive_message("unknown recipient message",
                             connection,
                             constants.unknow_recipient,
                             constants.expect_unknow_recipient
                             )
    
    print("User cannot send messages to not friends")
    connection.close()
