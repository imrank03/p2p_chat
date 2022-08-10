//! Protocol handler that handles messageing the remote at a regular period.

use futures::future::BoxFuture;
use futures::prelude::*;
use libp2p::core::{upgrade, InboundUpgrade, OutboundUpgrade, UpgradeInfo};
use libp2p::swarm::NegotiatedSubstream;
use std::{io, iter};

/// read and write msg demo

pub enum Success {
    OK,
}

#[derive(Default, Debug, Clone)]
pub struct MsgContent {
    pub data: Vec<u8>,
}

impl UpgradeInfo for MsgContent {
    type Info = &'static [u8];
    type InfoIter = iter::Once<Self::Info>;

    fn protocol_info(&self) -> Self::InfoIter {
        iter::once(b"/p2p/msg/1.0.0")
    }
}

impl InboundUpgrade<NegotiatedSubstream> for MsgContent {
    type Output = Vec<u8>;
    type Error = std::io::Error;
    type Future = BoxFuture<'static, Result<Self::Output, Self::Error>>;

    fn upgrade_inbound(self, socket: NegotiatedSubstream, _: Self::Info) -> Self::Future {
        //println!("upgrade_inbound");
        async move {
            let packet = recv(socket).await?;
            Ok(packet)
        }
        .boxed()
    }
}

impl OutboundUpgrade<NegotiatedSubstream> for MsgContent {
    type Output = Success;
    type Error = std::io::Error;
    type Future = BoxFuture<'static, Result<Self::Output, Self::Error>>;
    fn upgrade_outbound(self, socket: NegotiatedSubstream, _: Self::Info) -> Self::Future {
        //println!("upgrade_outbound {:?}",self.data);
        async move {
            send(socket, self.data).await?;
            Ok(Success::OK)
        }
        .boxed()
    }
}

pub async fn recv<S>(mut socket: S) -> io::Result<Vec<u8>>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let packet = upgrade::read_length_prefixed(&mut socket, 2048).await?;
    //println!("{:?}",std::str::from_utf8(&packet).unwrap());

    Ok(packet)
}
pub async fn send<S>(mut socket: S, data: Vec<u8>) -> io::Result<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    upgrade::write_length_prefixed(&mut socket, data).await?;
    socket.close().await?;
    Ok(socket)
}
