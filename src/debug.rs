//! Debugging helpers

use hdk::prelude::*;
use holo_hash::*;

#[macro_export]
macro_rules! zome_error {
   ($($arg:tt)*) => {
      {
         let msg = format!($($arg)*);
         Err(wasm_error!(WasmErrorInner::Guest(msg)))
      }
   }
}

/// Panic hook for zome debugging
pub fn zome_panic_hook(info: &std::panic::PanicInfo) {
   let mut msg = info.to_string();
   msg.push_str("\n\nPanic during zome call ");
   msg.push_str(&dump_context());
   error!("{}\n\n", &msg);
}

pub fn error<T>(reason: &str) -> ExternResult<T> {
   let msg = format!("{} ; Context: {}", reason, dump_context());
   let error = WasmError {
      file: String::new(),
      line: 0,
      error: WasmErrorInner::Guest(msg),
   };
   Err(error)
}


pub fn invalid(reason: &str) -> ExternResult<ValidateCallbackResult> {
   Ok(ValidateCallbackResult::Invalid(reason.to_string()))
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
            .map_err(|_| error::<T>("Deserializing zome call response failed").err().unwrap());
         res
      },
      ZomeCallResponse::Unauthorized(_, _, _, _) => error("Unauthorized call"),
      ZomeCallResponse::NetworkError(e) => error(&format!("NetworkError: {:?}", e)),
      ZomeCallResponse::CountersigningSession(e) => error(&format!("CountersigningSession: {:?}", e)),
   };
}

/// Shorten AgentPubKey for printing
pub fn snip(agent: &AgentPubKey) -> String {
   //format!("{:?}", agent)[12..24].to_string()
   //format!("{}", agent)[..12].to_string()
   let b64: AgentPubKeyB64 = AgentPubKeyB64::from(agent.clone());
   format!("{:?}", b64)[24..36].to_string()
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



