use super::worker;
use core::{fmt, future::Future, pin::Pin};
use libp2p::{multiaddr, Multiaddr, PeerId};
use smallvec::{smallvec, SmallVec};

pub struct NetworkBuilder {
    /// How to spawn background tasks. If you pass `None`, then a threads pool will be used by
    /// default.
    executor: Option<Box<dyn Fn(Pin<Box<dyn Future<Output = ()> + Send>>) + Send>>,

    /// Small string identifying the chain, in order to detect incompatible nodes earlier.
    chain_spec_protocol_id: SmallVec<[u8; 6]>,

    /// List of known bootnodes.
    boot_nodes: Vec<(PeerId, Multiaddr)>,
}

/// Creates a new prototype of the network.
pub fn builder() -> NetworkBuilder {
    NetworkBuilder {
        executor: None,
        chain_spec_protocol_id: smallvec![b's', b'u', b'p'],
        boot_nodes: Vec::new(),
    }
}

impl NetworkBuilder {
    pub fn with_executor(
        mut self,
        executor: Box<dyn Fn(Pin<Box<dyn Future<Output = ()> + Send>>) + Send>,
    ) -> Self {
        self.executor = Some(executor);
        self
    }

    /// Sets the list of bootstrap nodes to use.
    ///
    /// A **bootstrap node** is a node known from the network at startup.
    pub fn set_boot_nodes(&mut self, list: impl Iterator<Item = (PeerId, Multiaddr)>) {
        self.boot_nodes = list.collect();
    }

    /// Sets the list of bootstrap nodes to use.
    ///
    /// A **bootstrap node** is a node known from the network at startup.
    pub fn with_boot_nodes(mut self, list: impl Iterator<Item = (PeerId, Multiaddr)>) -> Self {
        self.set_boot_nodes(list);
        self
    }

    /// Sets the name of the chain to use on the network to identify incompatible peers earlier.
    pub fn set_chain_spec_protocol_id(&mut self, id: impl AsRef<[u8]>) {
        self.chain_spec_protocol_id = id.as_ref().into_iter().cloned().collect();
    }

    /// Sets the name of the chain to use on the network to identify incompatible peers earlier.
    pub fn with_chain_spec_protocol_id(mut self, id: impl AsRef<[u8]>) -> Self {
        self.set_chain_spec_protocol_id(id);
        self
    }

    /// Starts the networking.
    pub async fn build(self) -> worker::Network {
        worker::Network::start(worker::Config {
            known_addresses: self.boot_nodes,
            chain_spec_protocol_id: self.chain_spec_protocol_id,
        })
        .await
    }
}

/// Parses a string address and splits it into Multiaddress and PeerId, if
/// valid.
///
/// # Example
///
/// ```
/// # use sc_network::{Multiaddr, PeerId, config::parse_str_addr};
/// let (peer_id, addr) = parse_str_addr(
/// 	"/ip4/198.51.100.19/tcp/30333/p2p/QmSk5HQbn6LhUwDiNMseVUjuRYhEtYj4aUZ6WfWoGURpdV"
/// ).unwrap();
/// assert_eq!(peer_id, "QmSk5HQbn6LhUwDiNMseVUjuRYhEtYj4aUZ6WfWoGURpdV".parse::<PeerId>().unwrap());
/// assert_eq!(addr, "/ip4/198.51.100.19/tcp/30333".parse::<Multiaddr>().unwrap());
/// ```
///
pub fn parse_str_addr(addr_str: &str) -> Result<(PeerId, Multiaddr), ParseErr> {
    let addr: Multiaddr = addr_str.parse()?;
    parse_addr(addr)
}

/// Splits a Multiaddress into a Multiaddress and PeerId.
pub fn parse_addr(mut addr: Multiaddr) -> Result<(PeerId, Multiaddr), ParseErr> {
    let who = match addr.pop() {
        Some(multiaddr::Protocol::P2p(key)) => {
            PeerId::from_multihash(key).map_err(|_| ParseErr::InvalidPeerId)?
        }
        _ => return Err(ParseErr::PeerIdMissing),
    };

    Ok((who, addr))
}

/// Error that can be generated by `parse_str_addr`.
#[derive(Debug)]
pub enum ParseErr {
    /// Error while parsing the multiaddress.
    MultiaddrParse(multiaddr::Error),
    /// Multihash of the peer ID is invalid.
    InvalidPeerId,
    /// The peer ID is missing from the address.
    PeerIdMissing,
}

impl fmt::Display for ParseErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseErr::MultiaddrParse(err) => write!(f, "{}", err),
            ParseErr::InvalidPeerId => write!(f, "Peer id at the end of the address is invalid"),
            ParseErr::PeerIdMissing => write!(f, "Peer id is missing from the address"),
        }
    }
}

impl std::error::Error for ParseErr {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ParseErr::MultiaddrParse(err) => Some(err),
            ParseErr::InvalidPeerId => None,
            ParseErr::PeerIdMissing => None,
        }
    }
}

impl From<multiaddr::Error> for ParseErr {
    fn from(err: multiaddr::Error) -> ParseErr {
        ParseErr::MultiaddrParse(err)
    }
}