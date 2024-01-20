# Unknown recipient
unknow_recipient = '{"recipient":"unknown-lol","message":"dfe0d38c3a4b0a6b37a730591f86012495aac1ed60369084d4aa056edc83055901575d00e1e7a0ec43841f0fdb0c746d38b32d6fcb9d80a6d12020268475cea8b679e14dbb19a60e96ced8e4db79f4d59eaee9c0796fe091e7c00abb7d39c1204033065ac6d8628c84ea5b4edf80a21701a3aaee464649e6cadc08d931007d8e63bbaf5e4785b4f51e50b3afb4f7ccfe94b8517c69d2eeab144c92a47f41c7504bba309883d8cdbb155dd3dcc5e48483dc3c0f7c1234d255a6f53f09868d6c6719f801d9b4e393b99f4e2be76760a9ec798afae842e6659df7c520f9891532e1e51c3a96fe6e74652dc8e5c28776f6566e4fad7a4da4e6d79cb46f21c6d3dcfe","message_self_encrypted":"74e5a6912ed26010ab766fbcb6994a24bacc055416bcaf42a0d20650dbd04f7337753e7749a8f367588078b49ae981e7e25690377fb0d600b3ed3b03895dff4080c8d951764390b15ca2cd962517e82e6715f1e8be447bf0b3010b6b84fbbb3a91b94d05ce3147506be3e92bdbec8645985aba6bfd072a3f6cf4589317ebcc6a470d57ef2d8a839ae6cf95040a427aa2e22ccd6b4b70ce8e6bd4484be320fa04664dcd2d0b7015eec2c893c44908740f5f6313af44cf43887389449edb478bd688f124c59f49b00497ac52f4c7ccdef22f66623ab90e7e3e0ac7c2619e8d51d69e5163f5dc89562e7ecc1e957cf347cedf7210312bbd1f6e80db06d4efe478fb","message_signature":"705a8457d829c522f3c971ea5353e8433f8fb1d4cab3bc6e6a1e6f568ee9d9e386999e5c74c596ed2618bc3fff56ea3fb67c8447e7b00035bf9684903e7e52a68a1f460ba2e2ff515ee53262119d04d2607b8d6adc046a123277eb8a12adc5af6baa2a799ba54e084ebf3ec4fcc39c2be5c602bc234cc6ebc587c31778026602b34f3ac1db987cc160cb4b13b722a3bce89e785d67519855bb192e57ab970e80a12f1c99bc36bd2a31fb4cae716496de4f5ef3f3476009d16ab7e98494beb2e487fcc8dce33f536ccce033703d1a598094688b03135b7f093eb7c00677b08c4f38b72b1d357200a86184a351011363b0c1011280f022e51b130ef71d6e68d66a","message_self_encrypted_signature":"80f32a86044072f5ad637ca86be3a33ae837c7ee4b4b5ea2304d0edbdd125a7985d4da3308b4680b74280ce14c8f155a077645b9e27bc4635c255877c6abeb4f62eb609facb6f648bac3ecdd7b29e82c42a28fc23dfed078a702bc9a518df6d3951623fbc6a1b68c5011bf6ee28fb204fc95425aae345e2a892e00c195490d5e73f08b0f0ec378807a593eee6fbff0f321c0318b965daa02d47ba64a8f25e7ecf4bea7ca2ebdcb02eb47e62ac3e7cda023440bf71c2ac5d2ad52ab6fe8dc34f8ab87acf0a04cdd956176d894ba252c1b2efc1b8354a18fc9e03b4a32eac27dfde35dc53cf67ccdeb24c8748b84af236bb9f004211b44a2b668db8c9ba9caeda5"}'
expect_unknow_recipient = '{"message":"You are not befriended with unknown-lol","TYPE":"SOCKET_MESSAGE_ERROR"}'


# Deserialization test
deserialize = '{"this-is-not-known":"not-known-format"}'
expect_deserialize = '{"message":"Could not deserialize {\\"this-is-not-known\\":\\"not-known-format\\"}","TYPE":"SOCKET_MESSAGE_ERROR"}'


# Dual connection test
def get_testuser2_to_testuser1_message(recipient: str):
    return '{"recipient":"' +recipient + '","message":"374cd1a025ba4f9e8822036180eb5dac3d0a60e18237336e081d27d836fed82c5dbf6373726714e541b0a29986788ace88fc9f4ece0fbd61d1c64493102c8ef09913497accbb2e5f37d102b3dcafd4c9eefba3f874505d45c89568719862e001f3e932b06a51dc72847032d0defea5d433b0900d22b47b6c5136deb863a4dd03007e718fb241630aa7850fc8b60480b65d2263e39ec28b752544f2d38c79b98d4bff3fc18c55ec52df0a8c8023c7bf2c324562f20afde176e15b1b880cd3c79e80efa58e36a3ce5bba45d9c5a8ef842603e350f12218b3976248a5ab138fa562e0da88f48256fb1bd5716e0d3066dd819315dd059fa542c293454b35b05544c8","message_self_encrypted":"75f85551de84f04259afff64b65c4df9a7ce83ed883054798640abf9571db02b9a7abe86b81f8417fb2dea1174f93ea64858059a9b680622649d3c8fcb3af2fd628c2faef8b43d091f1ace2deaea1bad690fc0b8eae227ae1d575477144464809339cd48f7b6a8438ece0caeb7b1e336a5f96144c1cb11b7e398e8f8b7ba1b46cdcf0d7828cbe26a86e889528841e464a45a8e5d04866bf9777b94ecec09965854acab80861b3eb6403f0939f694a3f10fefb20b1a9c076abda78ab6f6d130b0e6ed7c1bab85d57833f8aa650beaec84ff6574f0a8279663684edc2ce1542142b3726c1f6ec4ea6f6ed5bba9a8e3128b02ef0108abe8eb0addf8ce21cf06d41e","message_signature":"74c192ffc8389879e4809e0b37c1c263346278405ee4e2ffee8240db47433d41e56ad3eb07cf4c99634b3cfb4e0aa22c6a85f2ac493fa2a8763489e489a342a53479ae8511997f826d544a243c6b381fa4a724565be61e4e8b516fe3350e771f7bd4356b495d2cb5616ca139a74df7ee6d3ea875b1a6f8d9515a17f95e0cca43404de6c68b92e45853539ea80989acd01db770bdeb927f6672bd1c7fd96b908be19167755f6776edfaf759d1033dfcd803fbfc79fac47c4da0cb0bdec9f6b64a165e2977d2be471e3bc446deb36a20e5cef17c4e6fd50cfc9a12b46aefc1ae7ce2dc595cf65bf71c775d8497ea04ffb45f000be16ddc2e6746624aa9f640aa37","message_self_encrypted_signature":"4d3f5ee22edb705f506938efc9e0b4cc91eeb042af3ad157a6331733e8ae22d9e081d8d48ce046f0bc9d9af65cfa9884bfa2b301061481c4e10505b58e22f7cf96d9c742742e62ac8e76f2af38c3ed4f5a42ac606a99133768ec7aeabab88e1cbe41f414265a94683065ae23fdd2ac4de55a9f5b38ff1150f70d9456166928107efe0d3dad0578890208b584a0fc7c8907d10694569fe5a2a04b276abe9320e27c7840320b4b1f0223e456d18047653af426a3fe9eb3380d4d5b230ffe12651c3a7dc4aa0738504f2d481e4b99b8737ba16cacdeccddfb1b9779f6709fa30d859ecb8a85052060a6bb48c90fdbb89e55dcbc213458fecb57f218ddae14afdbe3"}'
def get_expect_testuser2_to_testuser1_message(id: str, recipient: str, sender: str):
    return '{"recipient":"' +recipient + '","sender":"' +sender + '","message":"374cd1a025ba4f9e8822036180eb5dac3d0a60e18237336e081d27d836fed82c5dbf6373726714e541b0a29986788ace88fc9f4ece0fbd61d1c64493102c8ef09913497accbb2e5f37d102b3dcafd4c9eefba3f874505d45c89568719862e001f3e932b06a51dc72847032d0defea5d433b0900d22b47b6c5136deb863a4dd03007e718fb241630aa7850fc8b60480b65d2263e39ec28b752544f2d38c79b98d4bff3fc18c55ec52df0a8c8023c7bf2c324562f20afde176e15b1b880cd3c79e80efa58e36a3ce5bba45d9c5a8ef842603e350f12218b3976248a5ab138fa562e0da88f48256fb1bd5716e0d3066dd819315dd059fa542c293454b35b05544c8","message_signature":"74c192ffc8389879e4809e0b37c1c263346278405ee4e2ffee8240db47433d41e56ad3eb07cf4c99634b3cfb4e0aa22c6a85f2ac493fa2a8763489e489a342a53479ae8511997f826d544a243c6b381fa4a724565be61e4e8b516fe3350e771f7bd4356b495d2cb5616ca139a74df7ee6d3ea875b1a6f8d9515a17f95e0cca43404de6c68b92e45853539ea80989acd01db770bdeb927f6672bd1c7fd96b908be19167755f6776edfaf759d1033dfcd803fbfc79fac47c4da0cb0bdec9f6b64a165e2977d2be471e3bc446deb36a20e5cef17c4e6fd50cfc9a12b46aefc1ae7ce2dc595cf65bf71c775d8497ea04ffb45f000be16ddc2e6746624aa9f640aa37","message_self_encrypted":"75f85551de84f04259afff64b65c4df9a7ce83ed883054798640abf9571db02b9a7abe86b81f8417fb2dea1174f93ea64858059a9b680622649d3c8fcb3af2fd628c2faef8b43d091f1ace2deaea1bad690fc0b8eae227ae1d575477144464809339cd48f7b6a8438ece0caeb7b1e336a5f96144c1cb11b7e398e8f8b7ba1b46cdcf0d7828cbe26a86e889528841e464a45a8e5d04866bf9777b94ecec09965854acab80861b3eb6403f0939f694a3f10fefb20b1a9c076abda78ab6f6d130b0e6ed7c1bab85d57833f8aa650beaec84ff6574f0a8279663684edc2ce1542142b3726c1f6ec4ea6f6ed5bba9a8e3128b02ef0108abe8eb0addf8ce21cf06d41e","message_self_encrypted_signature":"4d3f5ee22edb705f506938efc9e0b4cc91eeb042af3ad157a6331733e8ae22d9e081d8d48ce046f0bc9d9af65cfa9884bfa2b301061481c4e10505b58e22f7cf96d9c742742e62ac8e76f2af38c3ed4f5a42ac606a99133768ec7aeabab88e1cbe41f414265a94683065ae23fdd2ac4de55a9f5b38ff1150f70d9456166928107efe0d3dad0578890208b584a0fc7c8907d10694569fe5a2a04b276abe9320e27c7840320b4b1f0223e456d18047653af426a3fe9eb3380d4d5b230ffe12651c3a7dc4aa0738504f2d481e4b99b8737ba16cacdeccddfb1b9779f6709fa30d859ecb8a85052060a6bb48c90fdbb89e55dcbc213458fecb57f218ddae14afdbe3","id":"' + id + '","TYPE":"SOCKET_MESSAGE_DIRECT"}'
def get_expect_testuser2_to_testuser1_receive(id: str, recipient: str, sender: str):
    return '{"recipient":"' +recipient + '","sender":"' +sender + '","message":"374cd1a025ba4f9e8822036180eb5dac3d0a60e18237336e081d27d836fed82c5dbf6373726714e541b0a29986788ace88fc9f4ece0fbd61d1c64493102c8ef09913497accbb2e5f37d102b3dcafd4c9eefba3f874505d45c89568719862e001f3e932b06a51dc72847032d0defea5d433b0900d22b47b6c5136deb863a4dd03007e718fb241630aa7850fc8b60480b65d2263e39ec28b752544f2d38c79b98d4bff3fc18c55ec52df0a8c8023c7bf2c324562f20afde176e15b1b880cd3c79e80efa58e36a3ce5bba45d9c5a8ef842603e350f12218b3976248a5ab138fa562e0da88f48256fb1bd5716e0d3066dd819315dd059fa542c293454b35b05544c8","message_signature":"74c192ffc8389879e4809e0b37c1c263346278405ee4e2ffee8240db47433d41e56ad3eb07cf4c99634b3cfb4e0aa22c6a85f2ac493fa2a8763489e489a342a53479ae8511997f826d544a243c6b381fa4a724565be61e4e8b516fe3350e771f7bd4356b495d2cb5616ca139a74df7ee6d3ea875b1a6f8d9515a17f95e0cca43404de6c68b92e45853539ea80989acd01db770bdeb927f6672bd1c7fd96b908be19167755f6776edfaf759d1033dfcd803fbfc79fac47c4da0cb0bdec9f6b64a165e2977d2be471e3bc446deb36a20e5cef17c4e6fd50cfc9a12b46aefc1ae7ce2dc595cf65bf71c775d8497ea04ffb45f000be16ddc2e6746624aa9f640aa37","message_self_encrypted":"75f85551de84f04259afff64b65c4df9a7ce83ed883054798640abf9571db02b9a7abe86b81f8417fb2dea1174f93ea64858059a9b680622649d3c8fcb3af2fd628c2faef8b43d091f1ace2deaea1bad690fc0b8eae227ae1d575477144464809339cd48f7b6a8438ece0caeb7b1e336a5f96144c1cb11b7e398e8f8b7ba1b46cdcf0d7828cbe26a86e889528841e464a45a8e5d04866bf9777b94ecec09965854acab80861b3eb6403f0939f694a3f10fefb20b1a9c076abda78ab6f6d130b0e6ed7c1bab85d57833f8aa650beaec84ff6574f0a8279663684edc2ce1542142b3726c1f6ec4ea6f6ed5bba9a8e3128b02ef0108abe8eb0addf8ce21cf06d41e","message_self_encrypted_signature":"4d3f5ee22edb705f506938efc9e0b4cc91eeb042af3ad157a6331733e8ae22d9e081d8d48ce046f0bc9d9af65cfa9884bfa2b301061481c4e10505b58e22f7cf96d9c742742e62ac8e76f2af38c3ed4f5a42ac606a99133768ec7aeabab88e1cbe41f414265a94683065ae23fdd2ac4de55a9f5b38ff1150f70d9456166928107efe0d3dad0578890208b584a0fc7c8907d10694569fe5a2a04b276abe9320e27c7840320b4b1f0223e456d18047653af426a3fe9eb3380d4d5b230ffe12651c3a7dc4aa0738504f2d481e4b99b8737ba16cacdeccddfb1b9779f6709fa30d859ecb8a85052060a6bb48c90fdbb89e55dcbc213458fecb57f218ddae14afdbe3","id":"' + id + '","TYPE":"SOCKET_MESSAGE_DIRECT"}'




def get_testuser1_to_testuser2_message(recipient: str):
    return '{"recipient":"' +recipient + '", "message":"990ce323ecd3a398ef0b43cd6a6a76584bece73c9b03d3f63c48c4217e63f6cc8e0ff0c5526392010959e1507dfe0263fbc450b66ab129de177c47d7b1d87851bcc86b4a8d6fcb5b7fe9e90b2862b373f664b8d332b1151552149c5633a7054d74b039899478ef5970e845420e4568e02c9f0829664f988c6607854ff0d960d297fefb8872982e71a3ad22e1543913f6473014cd3c95d3a5610765d4687467bbad14986e6996be652ba8efcadabe7ed634b42431e30543c0d2e4fe9f5e7012362684eb68b1acd45d5ec80ffc413b0a6353f8c45e118168762dc9e200d4f76771d7a34901245037559bb953888e748cbd5c1483e48ad9de007c64e4049f91e16e","message_self_encrypted":"4386a06a1eb4703d8d1e45e28af80cce10c3207b0693d625d5973a391c14c5cfc810255b68145aa6ba952222d83b27a0bb7c3bac24a0d6cdf14eab6d3645dcbda04c627149d1e22da19d4b2e2bbffc6869797b323367a58110ab83a23e6dde9efe5825703967262f5acc8b6e5be9deb526277ee0bd60d1de9b5a4528e6df9526e5e6c3f10ac0b7ea5ada5f7ad69f3629ab26c22ca853f6daf7e4b9d8af7634523094403e4af93a4acb23fb20e7a09f69eb44c10a120629b7985cc43a0445fb9fd4ee7a585302e3831f4581bd2572ea6746cb01bee080ad673e011caf2cd8ee0080c619e81728343fc8a97dbfe0499979fe757ddacf8e0e66f47c3a737fb04432","message_signature":"68257497716f7ee7be719810b881d113f34254c5363202c24bc2b437462bfc71838d16de5eddeaa82ff0b88c895bc86c916a6ab15eb2ae8e76d5d4f5d99c61a7c3d7707b60694f489936efc51867f3b75436f644ab6e68eabb4e0536c984775a876f8cf4017c303715b17bdf5ad31bdccab52110319b7cb1d644eae08d16139cf38b8d525eba917760b3d7b7e1ef32e7a2653d973d73c20544296f37ab24e7c89c6d28ba44e97db17cfe60719da9b0970ec0c1a28f6c0ac5e1ca310d7e8ba497dcdb347a70452e25d35b85d1a7cffe95d9203c36fd16f10b27f9eaeada404042d39903f47be1c6fff7b348531ef2eb072aa2d3a86e6434161900e00e888bd811","message_self_encrypted_signature":"8f3cf62354640566b05cfc22a854d9837ce8ac11f3d5f4f69aa8def2dd8f0e10118898fcc8b5abe1452335993cd97d31fa202d0712c55897f97be0fa7d4bc32d966fece48d01c4c29efd7fba76e59ba58193304981b3ac6aa663215076fd758e3f25de0b5fd1b77ee269c2c2dc7394795b681f72bde293f31d0237dc13013e5cd235f2c3afc6fad693d0d1cc3cb61e52afbebc8316fbe67114f8c49d1b1c60b924282948a6b4291ffdc4a0d29d4100797aa7a674f5d6752348f5efff34e4e00e92873a7eef210006824c69d579a4d0b2351bbfe822a87b1138cd52e713f212d7a422d2db0fbab836237cf13d703e35720ddb6686dc4f6f18e31b773a4daffadb"}'
def get_expect_testuser1_to_testuser2_message(id: str, recipient: str, sender: str):
    return '{"recipient":"' +recipient + '","sender":"' +sender + '","message":"990ce323ecd3a398ef0b43cd6a6a76584bece73c9b03d3f63c48c4217e63f6cc8e0ff0c5526392010959e1507dfe0263fbc450b66ab129de177c47d7b1d87851bcc86b4a8d6fcb5b7fe9e90b2862b373f664b8d332b1151552149c5633a7054d74b039899478ef5970e845420e4568e02c9f0829664f988c6607854ff0d960d297fefb8872982e71a3ad22e1543913f6473014cd3c95d3a5610765d4687467bbad14986e6996be652ba8efcadabe7ed634b42431e30543c0d2e4fe9f5e7012362684eb68b1acd45d5ec80ffc413b0a6353f8c45e118168762dc9e200d4f76771d7a34901245037559bb953888e748cbd5c1483e48ad9de007c64e4049f91e16e","message_signature":"68257497716f7ee7be719810b881d113f34254c5363202c24bc2b437462bfc71838d16de5eddeaa82ff0b88c895bc86c916a6ab15eb2ae8e76d5d4f5d99c61a7c3d7707b60694f489936efc51867f3b75436f644ab6e68eabb4e0536c984775a876f8cf4017c303715b17bdf5ad31bdccab52110319b7cb1d644eae08d16139cf38b8d525eba917760b3d7b7e1ef32e7a2653d973d73c20544296f37ab24e7c89c6d28ba44e97db17cfe60719da9b0970ec0c1a28f6c0ac5e1ca310d7e8ba497dcdb347a70452e25d35b85d1a7cffe95d9203c36fd16f10b27f9eaeada404042d39903f47be1c6fff7b348531ef2eb072aa2d3a86e6434161900e00e888bd811","message_self_encrypted":"4386a06a1eb4703d8d1e45e28af80cce10c3207b0693d625d5973a391c14c5cfc810255b68145aa6ba952222d83b27a0bb7c3bac24a0d6cdf14eab6d3645dcbda04c627149d1e22da19d4b2e2bbffc6869797b323367a58110ab83a23e6dde9efe5825703967262f5acc8b6e5be9deb526277ee0bd60d1de9b5a4528e6df9526e5e6c3f10ac0b7ea5ada5f7ad69f3629ab26c22ca853f6daf7e4b9d8af7634523094403e4af93a4acb23fb20e7a09f69eb44c10a120629b7985cc43a0445fb9fd4ee7a585302e3831f4581bd2572ea6746cb01bee080ad673e011caf2cd8ee0080c619e81728343fc8a97dbfe0499979fe757ddacf8e0e66f47c3a737fb04432","message_self_encrypted_signature":"8f3cf62354640566b05cfc22a854d9837ce8ac11f3d5f4f69aa8def2dd8f0e10118898fcc8b5abe1452335993cd97d31fa202d0712c55897f97be0fa7d4bc32d966fece48d01c4c29efd7fba76e59ba58193304981b3ac6aa663215076fd758e3f25de0b5fd1b77ee269c2c2dc7394795b681f72bde293f31d0237dc13013e5cd235f2c3afc6fad693d0d1cc3cb61e52afbebc8316fbe67114f8c49d1b1c60b924282948a6b4291ffdc4a0d29d4100797aa7a674f5d6752348f5efff34e4e00e92873a7eef210006824c69d579a4d0b2351bbfe822a87b1138cd52e713f212d7a422d2db0fbab836237cf13d703e35720ddb6686dc4f6f18e31b773a4daffadb","id":"' + id + '","TYPE":"SOCKET_MESSAGE_DIRECT"}'
def get_expect_testuser1_to_testuser2_receive(id: str, recipient: str, sender: str):
    return '{"recipient":"' +recipient + '","sender":"' +sender + '","message":"990ce323ecd3a398ef0b43cd6a6a76584bece73c9b03d3f63c48c4217e63f6cc8e0ff0c5526392010959e1507dfe0263fbc450b66ab129de177c47d7b1d87851bcc86b4a8d6fcb5b7fe9e90b2862b373f664b8d332b1151552149c5633a7054d74b039899478ef5970e845420e4568e02c9f0829664f988c6607854ff0d960d297fefb8872982e71a3ad22e1543913f6473014cd3c95d3a5610765d4687467bbad14986e6996be652ba8efcadabe7ed634b42431e30543c0d2e4fe9f5e7012362684eb68b1acd45d5ec80ffc413b0a6353f8c45e118168762dc9e200d4f76771d7a34901245037559bb953888e748cbd5c1483e48ad9de007c64e4049f91e16e","message_signature":"68257497716f7ee7be719810b881d113f34254c5363202c24bc2b437462bfc71838d16de5eddeaa82ff0b88c895bc86c916a6ab15eb2ae8e76d5d4f5d99c61a7c3d7707b60694f489936efc51867f3b75436f644ab6e68eabb4e0536c984775a876f8cf4017c303715b17bdf5ad31bdccab52110319b7cb1d644eae08d16139cf38b8d525eba917760b3d7b7e1ef32e7a2653d973d73c20544296f37ab24e7c89c6d28ba44e97db17cfe60719da9b0970ec0c1a28f6c0ac5e1ca310d7e8ba497dcdb347a70452e25d35b85d1a7cffe95d9203c36fd16f10b27f9eaeada404042d39903f47be1c6fff7b348531ef2eb072aa2d3a86e6434161900e00e888bd811","message_self_encrypted":"4386a06a1eb4703d8d1e45e28af80cce10c3207b0693d625d5973a391c14c5cfc810255b68145aa6ba952222d83b27a0bb7c3bac24a0d6cdf14eab6d3645dcbda04c627149d1e22da19d4b2e2bbffc6869797b323367a58110ab83a23e6dde9efe5825703967262f5acc8b6e5be9deb526277ee0bd60d1de9b5a4528e6df9526e5e6c3f10ac0b7ea5ada5f7ad69f3629ab26c22ca853f6daf7e4b9d8af7634523094403e4af93a4acb23fb20e7a09f69eb44c10a120629b7985cc43a0445fb9fd4ee7a585302e3831f4581bd2572ea6746cb01bee080ad673e011caf2cd8ee0080c619e81728343fc8a97dbfe0499979fe757ddacf8e0e66f47c3a737fb04432","message_self_encrypted_signature":"8f3cf62354640566b05cfc22a854d9837ce8ac11f3d5f4f69aa8def2dd8f0e10118898fcc8b5abe1452335993cd97d31fa202d0712c55897f97be0fa7d4bc32d966fece48d01c4c29efd7fba76e59ba58193304981b3ac6aa663215076fd758e3f25de0b5fd1b77ee269c2c2dc7394795b681f72bde293f31d0237dc13013e5cd235f2c3afc6fad693d0d1cc3cb61e52afbebc8316fbe67114f8c49d1b1c60b924282948a6b4291ffdc4a0d29d4100797aa7a674f5d6752348f5efff34e4e00e92873a7eef210006824c69d579a4d0b2351bbfe822a87b1138cd52e713f212d7a422d2db0fbab836237cf13d703e35720ddb6686dc4f6f18e31b773a4daffadb","id":"' + id + '","TYPE":"SOCKET_MESSAGE_DIRECT"}'
