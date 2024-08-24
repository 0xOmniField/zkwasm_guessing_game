//import initHostBind, * as hostbind from "./wasmbind/hostbind.js";
import { ZKWasmAppRpc } from "zkwasm-ts-server";
let account = "12345";
const rpc = new ZKWasmAppRpc("http://localhost:3000");
//const rpc = new ZKWasmAppRpc("http://114.119.187.224:8085");
async function main() {
    try {
        let state = await rpc.queryState(account);
        let data = JSON.parse(state.data);
        console.log("player info:", data);
    }
    catch (e) {
        console.log(e);
    }
    //let config = await rpc.query_config();
    //console.log("config", config);
}
main();
//# sourceMappingURL=query.js.map