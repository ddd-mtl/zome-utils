//! Debugging helpers

use hdk::prelude::*;
use crate as zome_utils;



#[macro_export]
macro_rules! zome_error {
   ($($arg:tt)*) => { {
         let line_number = line!();
         let file_name = file!().to_string();
         let reason = format!($($arg)*);
         let msg = format!("{} ; Context: {}", reason, zome_utils::dump_context());
         let error = WasmError {
            file: file_name,
            line: line_number,
            error: WasmErrorInner::Guest(msg),
         };
         Err(error)
      }
   }
}


///
pub fn error<T>(reason: &str) -> ExternResult<T> {
   let msg = format!("{} ; Context: {}", reason, dump_context());
   let error = WasmError {
      file: String::new(),
      line: 0,
      error: WasmErrorInner::Guest(msg),
   };
   Err(error)
}


///
pub fn invalid(reason: &str) -> ExternResult<ValidateCallbackResult> {
   Ok(ValidateCallbackResult::Invalid(reason.to_string()))
}


/// Return zome context as String
pub fn dump_context() -> String {
   let mut msg = String::new();
   let maybe_zome_info = zome_info();
   if let Ok(zome_info) = maybe_zome_info {
      let maybe_call_info = call_info();
      if let Ok(call_info) = maybe_call_info {
         let provenance = snip(&call_info.provenance);
         msg.push_str(&format!("'{}::{}()' by {} ",
                               zome_info.name, call_info.function_name, provenance));
      }
   }
   let maybe_agent_info = agent_info();
   if let Ok(agent_info) = maybe_agent_info {
      msg.push_str(&format!("in chain of {}", snip(&agent_info.agent_latest_pubkey)));
   }
   msg
}


/// Panic hook for zome debugging
pub fn zome_panic_hook(info: &std::panic::PanicInfo) {
   let mut msg = "\n\nPanic during zome call ".to_owned();
   msg.push_str(&dump_context());
   msg.push_str("\n\n");
   msg.push_str(&info.to_string());
   error!("{}\n\n", &msg);
}



/// Convert ZomeCallResponse to ExternResult
pub fn decode_response<T>(response: ZomeCallResponse) -> ExternResult<T>
   where
      //T: for<'de> serde::Deserialize<'de> + std::fmt::Debug,
      T: serde::de::DeserializeOwned + std::fmt::Debug
{
   return match response {
      ZomeCallResponse::Ok(output) => {
         let res = output
            .decode()
            .map_err(|_| error::<T>("Deserializing response output failed").err().unwrap());
         res
      },
      ZomeCallResponse::Unauthorized(auth, _, _, fn_name, _) => zome_error!("Unauthorized call to {}(): {:?}", fn_name, auth),
      ZomeCallResponse::NetworkError(e) => zome_error!("NetworkError: {:?}", e),
      ZomeCallResponse::CountersigningSession(e) => zome_error!("CountersigningSession: {:?}", e),
   };
}


/// Shorten AgentPubKey for printing
pub fn snip(agent: &AgentPubKey) -> String {
   //format!("{:?}", agent)[12..24].to_string()
   format!("{}", agent)[5..13].to_string()
   //let b64: AgentPubKeyB64 = AgentPubKeyB64::from(agent.clone());
   //format!("{:?}", b64)[5..13].to_string()
}
