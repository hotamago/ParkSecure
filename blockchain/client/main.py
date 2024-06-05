from typing import Optional
import uvicorn

from fastapi import FastAPI, Body, Depends, HTTPException,  File, UploadFile
from fastapi.middleware.cors import CORSMiddleware
from config import *
from pydantic import BaseModel
import json

# import data
from hotaSolana.hotaSolanaDataBase import *
from hotaSolana.hotaSolanaData import *
from hotaSolana.bs58 import bs58

from baseAPI import *

description = """
ChimichangApp API helps you do awesome stuff. 🚀

## Price

caculate in lamports / second

## Time

caculate in seconds
"""

app = FastAPI(title="Solana API",
              description=description,
              summary="This is a Solana API",
              version="v2.0",
              contact={
                  "name": "Hotamago Master",
                  "url": "https://www.linkedin.com/in/hotamago/",
              })

origins = ["*"]

app.add_middleware(
    CORSMiddleware,
    allow_origins=origins,
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Solana Client
client = HotaSolanaRPC(programId, False, "devnet")

# Solana instruction data
@BaseStructClass
class Coordinates:
    latitude=HotaFloat64()
    longitude=HotaFloat64()

@BaseInstructionDataClass(name="init_parking_area")
class ParkingAreaInitInstruction:
    coordinates=Coordinates()
    name=HotaStringUTF16(32)
    address=HotaStringUTF16(64)
    info=HotaStringUTF16(256)
    price=HotaUint32()
    secret_key=HotaHex(32)
    seed_random=HotaHex(8)

@BaseInstructionDataClass(name="update_parking_area")
class ParkingAreaUpdateInstruction:
    address=HotaStringUTF16(64)
    info=HotaStringUTF16(256)
    price=HotaUint32()
    secret_key=HotaHex(32)

@BaseInstructionDataClass(name="hide_parking_area")
class HideParkingAreaInstruction:
    secret_key=HotaHex(32)
    time_to_hide=HotaUint32()

# Solana account data
@BaseStructClass
class ParkingAreaData:
    owner=HotaPublicKey()
    user=HotaPublicKey()
    coordinates=Coordinates()
    name=HotaStringUTF16(32)
    address=HotaStringUTF16(64)
    info=HotaStringUTF16(256)
    price=HotaUint32()
    expired_time=HotaUint64()
    secret_key=HotaHex(32)

##### Router

# init_parking_area
class CoordinatesModel(BaseModel):
    latitude: float
    longitude: float
class InitParkingModel(BaseModel):
    coordinates: CoordinatesModel
    name: str
    address: str
    info: str
    price: int
    secret_key: str

@app.post("/init-parking-area")
async def init_parking_area(
    owner_private_key: str,
    initParkingModel: InitParkingModel,
):
    def fun():
        owner_keypair = makeKeyPair(owner_private_key)

        instruction_data = ParkingAreaInitInstruction()
        instruction_data.get("coordinates").get("latitude").object2struct(initParkingModel.coordinates.latitude)
        instruction_data.get("coordinates").get("longitude").object2struct(initParkingModel.coordinates.longitude)
        instruction_data.get("name").object2struct(initParkingModel.name)
        instruction_data.get("address").object2struct(initParkingModel.address)
        instruction_data.get("info").object2struct(initParkingModel.info)
        instruction_data.get("price").object2struct(initParkingModel.price)
        instruction_data.get("secret_key").deserialize(hash256(initParkingModel.secret_key))
        instruction_data.get("seed_random").random()

        parking_area_pubkey = findProgramAddress(createBytesFromArrayBytes(
            owner_keypair.public_key.byte_value,
            "parking_area".encode("utf-8"),
            bytes(instruction_data.get("seed_random").serialize()),
        ), client.program_id)

        instruction_address = client.send_transaction(
            instruction_data,
            [
                makePublicKey(sysvar_clock),
                makeKeyPair(payerPrivateKey).public_key,
                owner_keypair.public_key,
                parking_area_pubkey,
                makePublicKey(sysvar_rent),
                makePublicKey(system_program),
            ],
            [
                makeKeyPair(payerPrivateKey),
                owner_keypair,
            ],
            fee_payer=makeKeyPair(payerPrivateKey).public_key
        )

        return {
            "instruction_address": instruction_address,
            "parking_area_public_key": bs58.encode(parking_area_pubkey.byte_value),
        }

    return make_response_auto_catch(fun)

# update_parking_area
class UpdateParkingModel(BaseModel):
    address: str
    info: str
    price: int
    secret_key: str

@app.post("/update-parking-area")
async def update_parking_area(
    owner_private_key: str,
    parking_area_public_key: str,
    updateParkingModel: UpdateParkingModel,
):
    def fun():
        owner_keypair = makeKeyPair(owner_private_key)
        parking_area_pubkey = PublicKey(parking_area_public_key)

        instruction_data = ParkingAreaUpdateInstruction()
        instruction_data.get("address").object2struct(updateParkingModel.address)
        instruction_data.get("info").object2struct(updateParkingModel.info)
        instruction_data.get("price").object2struct(updateParkingModel.price)
        instruction_data.get("secret_key").deserialize(hash256(updateParkingModel.secret_key))

        instruction_address = client.send_transaction(
            instruction_data,
            [
                makePublicKey(sysvar_clock),
                makeKeyPair(payerPrivateKey).public_key,
                owner_keypair.public_key,
                parking_area_pubkey,
                makePublicKey(sysvar_rent),
                makePublicKey(system_program),
            ],
            [
                makeKeyPair(payerPrivateKey),
                owner_keypair,
            ],
            fee_payer=makeKeyPair(payerPrivateKey).public_key
        )

        return {
            "instruction_address": instruction_address,
            "parking_area_public_key": bs58.encode(parking_area_pubkey.byte_value),
        }
    
    return make_response_auto_catch(fun)

# hide_parking_area
class HideParkingModel(BaseModel):
    secret_key: str
    time_to_hide: int

@app.post("/hide-parking-area")
async def hide_parking_area(
    owner_private_key: str,
    parking_area_public_key: str,
    hideParkingModel: HideParkingModel,
):
    def fun():
        owner_keypair = makeKeyPair(owner_private_key)
        parking_area_pubkey = PublicKey(parking_area_public_key)

        instruction_data = HideParkingAreaInstruction()
        instruction_data.get("secret_key").deserialize(hash256(hideParkingModel.secret_key))
        instruction_data.get("time_to_hide").object2struct(hideParkingModel.time_to_hide)

        instruction_address = client.send_transaction(
            instruction_data,
            [
                makePublicKey(sysvar_clock),
                makeKeyPair(payerPrivateKey).public_key,
                owner_keypair.public_key,
                parking_area_pubkey,
                # makePublicKey(sysvar_rent),
                makePublicKey(system_program),
            ],
            [
                makeKeyPair(payerPrivateKey),
                owner_keypair,
            ],
            fee_payer=makeKeyPair(payerPrivateKey).public_key
        )

        return {
            "instruction_address": instruction_address,
            "parking_area_public_key": bs58.encode(parking_area_pubkey.byte_value),
            "time_to_hide": hideParkingModel.time_to_hide,
        }
    
    return make_response_auto_catch(fun)

#### Common function1
@app.post("/convert-keypair-to-private-key")
async def convert_keypair_to_private_key(file: UploadFile):
    # Bytes to string
    result = file.file.read()
    keypair_json = json.loads(result)
    keypair_bytes = bytes(keypair_json)
    return {
        "public_key": bs58.encode(keypair_bytes[32:]),
        "private_key": bs58.encode(keypair_bytes),
    }

@app.get("/get-parking-area-info")
async def get_parking_area_info(public_key: str):
    return make_response_auto_catch(lambda: client.get_account_info(PublicKey(public_key)))

@app.get("/get-parking_area-data")
async def get_parking_area_data(public_key: str):
    return make_response_auto_catch(lambda: client.get_account_data(PublicKey(public_key), ParkingAreaData, [8, 4]))

@app.get("/get-balance")
async def get_balance(public_key: str):
    return make_response_auto_catch(client.get_balance(public_key))

@app.post("/airdrop")
async def airdrop(public_key: str, amount: int = 1):
    return make_response_auto_catch(client.drop_sol(public_key, amount))

# Run
if __name__ == "__main__":
    uvicorn.run(app, host="0.0.0.0", port=openPortAPI)
