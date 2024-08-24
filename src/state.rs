use crate::{game::RoundInfo, player::CombatPlayer, settlement::SettlementInfo};
use serde::Serialize;
use sha2::{Digest, Sha256};
use zkwasm_rest_abi::WithdrawInfo;
use zkwasm_rust_sdk::{require, PoseidonHasher};
use crate::game::Game;

const TIMETICK: u32 = 0;
const COMMITCARDS: u32 = 1;
const PROVIDECARDS: u32 = 2;
const WITHDRAW: u32 = 3;
const DEPOSIT: u32 = 4;

const GUESS: u32 = 5;


pub struct Transaction {
    pub command: u32,
    pub data: [u64; 3],
    pub old_params: Vec<u64>,
}

const ERROR_PLAYER_NOT_FOUND: u32 = 1;

static mut NUMBER_REAL: u64 = 0 ;

impl Transaction {
    pub fn decode_error(e: u32) -> &'static str {
        match e {
            ERROR_PLAYER_NOT_FOUND => "PlayerNotFound",
            _ => "Unknown"
        }
    }

    pub fn decode(params: [u64; 4], old_params: Vec<u64>) -> Self {
        let command = (params[0] & 0xffffffff) as u32;
        Transaction {
            command,
            data: [params[1], params[2], params[3]],
            old_params
        }
    }

    pub fn deposit(&self) -> u32 {
        let pid = [self.data[0], self.data[1]];
        let mut player = CombatPlayer::get_from_pid(&pid);
        let balance = self.data[3];
        match player.as_mut() {
            None => {
                let player = CombatPlayer::new_from_pid(pid);
                player.store();
            }
            Some(player) => {
                player.data.balance += balance;
                player.store();
            }
        }
        0
    }

    pub fn withdraw(&self, pkey: &[u64; 4]) -> u32 {
        let mut player = CombatPlayer::get_from_pid(&CombatPlayer::pkey_to_pid(pkey));
        match player.as_mut() {
            None => ERROR_PLAYER_NOT_FOUND,
            Some(player) => {
                let amount = self.data[0] & 0xffffffff;
                unsafe { require(player.data.balance >= amount) };
                let withdrawinfo = WithdrawInfo::new(&self.data);
                SettlementInfo::append_settlement(withdrawinfo);
                player.data.balance -= amount;
                player.store();
                0
            }
        }
    }

    pub fn get_num(&self) -> u64 {
        let mut hasher =  PoseidonHasher::new();

        let sigx_slice = &self.old_params[12..16];

        // 将 u64 数值转换为字节表示
        let mut byte_data = vec![];
        for &num in sigx_slice {
            byte_data.extend(&num.to_le_bytes());
        }
        // 将字节表示转换为字符串
        let sigx_string = hex::encode(byte_data);
        zkwasm_rust_sdk::dbg!("====== sigx_string {:?} \n", sigx_string);
        let key = "sigx";
        for d in sigx_slice {
            hasher.update(*d);
        }

        let result = hasher.finalize();
        let hash_bytes: [u8; 32] = unsafe { std::mem::transmute(result) };

        let hash_str = hex::encode(hash_bytes); // 将字节数组转换为十六进制字符串
        zkwasm_rust_sdk::dbg!("====== hash {:?} \n", hash_str);

        let hash_integer = result[0] ^ result[1] ^ result[2] ^ result[3];

        // 生成介于 1 和 n 之间的随机数
        let random_number = (hash_integer % 100) + 1;
        zkwasm_rust_sdk::dbg!("====== num {:?} \n", random_number);
        random_number
    }

    pub fn guess(&self, pkey: &[u64; 4]) -> u32 {
        if 0 == unsafe { NUMBER_REAL } {
            unsafe { NUMBER_REAL = self.get_num(); }
        }
        // self.get_num();
        let pid = CombatPlayer::pkey_to_pid(pkey);
        zkwasm_rust_sdk::dbg!("====== player is {:?} \n",pid);
        let mut player = CombatPlayer::get_from_pid(&pid);
        zkwasm_rust_sdk::dbg!("player is none {}\n", {player.is_none()});
        let mut player = match player {
            None => {
                zkwasm_rust_sdk::dbg!("====== player is none \n");
                CombatPlayer::new_from_pid(CombatPlayer::pkey_to_pid(pkey))
            }
            Some(player) => {
                let player_info =  serde_json::to_string(
                    &player,
                )
                    .unwrap();
                zkwasm_rust_sdk::dbg!("====== player11 {:?} \n",player_info);
                player
            }
        };
        let player_info =  serde_json::to_string(
            &player,
        )
            .unwrap();
        zkwasm_rust_sdk::dbg!("====== player {:?} \n",player_info);

        let state = unsafe { &mut STATE };

        if self.data[0] == unsafe { NUMBER_REAL } {
            player.data.last_result = 0;
            player.data.balance += 10;
            state.game.last_result = 0;
            unsafe { NUMBER_REAL = self.get_num(); }
        } else if self.data[0] < unsafe { NUMBER_REAL } {
            player.data.last_result = 1;
            state.game.last_result = 1;
        } else if self.data[0] > unsafe { NUMBER_REAL } {
            player.data.last_result = 2;
            state.game.last_result = 2;
        }
        player.store();
        0
    }

    pub fn process(&self, pid: &[u64; 4]) -> u32 {
        if self.command == TIMETICK {
            let state = unsafe { &mut STATE };
            state.counter += 1;
            state.game.settle();
            0
        } else if self.command == GUESS {
            self.guess(pid)
        }
        else if self.command == WITHDRAW {
            self.withdraw(pid)
        } else if self.command == DEPOSIT {
            self.deposit()
        } else {
            unreachable!()
        }
    }
}

#[derive(Serialize)]
pub struct State {
    counter: u64,
    game: Game,
}

pub static mut STATE: State = State {
    counter: 0,
    game: Game {
        total_dps: 0,
        progress: 0,
        target: 0,
        last_round_info: RoundInfo {
            locked_dps: 0,
            locked_rewards: 0,
        },
        last_result: 0,
    },
};

impl State {
    pub fn initialize() {}

    pub fn preempt() -> bool {
        return false;
    }

    pub fn get_state(_pid: Vec<u64>) -> String {
        serde_json::to_string(unsafe { &STATE }).unwrap()
    }

    pub fn store(&self) {}

    pub fn flush_settlement() -> Vec<u8> {
        let data = SettlementInfo::flush_settlement();
        unsafe { STATE.store() };
        data
    }
}
