use super::*;
use nt::state::State;

use std::sync::{Arc, Mutex};

use bytes::{Bytes, Buf, IntoBuf};
use leb128::LEB128Read;

pub enum RPCExecutionBody {
    V1(RPCV1ExecuteBody),
    V0(RPCV0ExecuteBody)
}

pub enum RPCResponseBody {
    V1(RPCV1ResponseBody),
    V0(RPCV0ResponseBody),
}

impl ClientMessage for RPCExecutionBody {
    fn encode(&self, buf: &mut BytesMut) {
        match *self {
            RPCExecutionBody::V0(ref v0) => v0.encode(buf),
            RPCExecutionBody::V1(ref v1) => v1.encode(buf),
        }
    }
}

impl RPCResponseBody {
    pub fn decode(mut buf: &mut Buf, state: &Arc<Mutex<State>>) -> Result<(Self, usize), ::failure::Error> {
        let mut bytes_read = 0;
        unimplemented!()
    }
}

//impl ServerMessage for RPCResponseBody {
//    fn decode(buf: &mut Buf) -> (Option<Self>, usize) {
//        let mut bytes_read = 0;
//
//    }
//}

//#[derive(ServerMessage)]
pub struct RPCV0ResponseBody {
    pub bytes: Vec<u8>
}

//#[derive(ServerMessage)]
pub struct RPCV1ResponseBody {
    pub results: Vec<RpcResult>,
}

#[derive(ClientMessage, new)]
pub struct RPCV1ExecuteBody {
    parameters: Vec<Parameter>,
}

#[derive(ClientMessage, new)]
pub struct RPCV0ExecuteBody {
    bytes: Vec<u8>
}

#[derive(Debug, Clone, PartialEq)]
pub struct RPCDefinitionData {
    version: u8,
    procedure_name: String,
    parameters_size: usize,
    parameters: Vec<Parameter>,
    result_size: usize,
    results: Vec<RpcResult>,
}

#[derive(Debug, Clone, PartialEq, ClientMessage)]
pub struct Parameter {
    parameter_type: EntryType,
    parameter_name: String,
    parameter_default: EntryValue,
}

#[derive(Debug, Clone, PartialEq, ServerMessage, ClientMessage)]
pub struct RpcResult {
    result_type: EntryType,
    result_name: String,
}

impl ServerMessage for Parameter {
    fn decode(buf: &mut Buf) -> Result<(Self, usize), ::failure::Error> {
        let mut bytes_read = 0;
        let parameter_type = {
            let (parameter_type, bytes) = EntryType::decode(buf)?;
            bytes_read += bytes;
            parameter_type
        };

        let parameter_name = {
            let (parameter_name, bytes) = String::decode(buf)?;
            bytes_read += bytes;
            parameter_name
        };

        let (parameter_default, bytes_read_entry) = parameter_type.get_entry(buf)?;
        bytes_read += bytes_read_entry;

        Ok((Parameter {
            parameter_type,
            parameter_name,
            parameter_default,
        }, bytes_read))
    }
}

impl ServerMessage for RPCDefinitionData {
    fn decode(mut buf: &mut Buf) -> ::std::result::Result<(Self, usize), ::failure::Error> {
        let mut bytes_read = 0;

        let mut buf = {
            let (len, bytes) = buf.read_unsigned()?;

            let len = len as usize;

            bytes_read += bytes;

            let mut slice = vec![0u8; len];
            buf.copy_to_slice(&mut slice[..]);
            Bytes::from(slice).into_buf()
        };

        let ver = buf.read_u8()?; // RPC Version
        bytes_read += 1;

        if ver == 0x00 {
            return Ok((RPCDefinitionData { version: ver, procedure_name: "".to_string(), result_size: 0,
                parameters_size: 0, results: Vec::new(), parameters: Vec::new() }, bytes_read));
        }

        let name = {
            let (s, bytes) = String::decode(&mut buf)?;
            bytes_read += bytes;
            s
        };

        let num_params = buf.read_u8()? as usize;
        bytes_read += 1;

        let mut params = Vec::with_capacity(num_params);

        for _ in 0..num_params {
            let (param, size) = Parameter::decode(&mut buf)?;
            bytes_read += size;
            params.push(param);
        }


        let num_outputs = buf.read_u8()? as usize;
        bytes_read += 1;

        let mut outputs = Vec::with_capacity(num_outputs);

        for _ in 0..num_outputs {
            let (output, size) = RpcResult::decode(&mut buf)?;
            bytes_read += size;
            outputs.push(output);
        }

        Ok((RPCDefinitionData {
            version: 0x01,
            procedure_name: name,
            parameters_size: num_params,
            parameters: params,
            result_size: num_outputs,
            results: outputs,
        }, bytes_read))
    }
}