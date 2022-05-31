use sp_core::{Pair, Public, sr25519, H160, Bytes};
use dust_runtime::{
	AccountId, CurrencyId,
	BabeConfig, BalancesConfig, GenesisConfig, SudoConfig, SystemConfig,
	IndicesConfig, EVMConfig, StakingConfig, SessionConfig, AuthorityDiscoveryConfig,
	WASM_BINARY,
	TokenSymbol, TokensConfig, DUST,
	StakerStatus,
	ImOnlineId, AuthorityDiscoveryId,
	MaxNativeTokenExistentialDeposit,
	get_all_module_accounts,
	opaque::SessionKeys,
};
use sp_consensus_babe::AuthorityId as BabeId;
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::traits::{IdentifyAccount};
use sc_service::{ChainType, Properties};
use sc_telemetry::TelemetryEndpoints;

use sp_std::{collections::btree_map::BTreeMap, str::FromStr};
use sc_chain_spec::ChainSpecExtension;

use serde::{Deserialize, Serialize};

use hex_literal::hex;
use sp_core::{crypto::UncheckedInto, bytes::from_hex};

use dust_primitives::{AccountPublic, Balance, Nonce};
use dust_runtime::BABE_GENESIS_EPOCH_CONFIG;

// The URL for the telemetry server.
const TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Node `ChainSpec` extensions.
///
/// Additional parameters for some Substrate core modules,
/// customizable from the chain spec.
#[derive(Default, Clone, Serialize, Deserialize, ChainSpecExtension)]
#[serde(rename_all = "camelCase")]
pub struct Extensions {
	/// Block numbers with known hashes.
	pub fork_blocks: sc_client_api::ForkBlocks<dust_primitives::Block>,
	/// Known bad block hashes.
	pub bad_blocks: sc_client_api::BadBlocks<dust_primitives::Block>,
}

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig, Extensions>;

fn get_session_keys(
	grandpa: GrandpaId,
	babe: BabeId,
	im_online: ImOnlineId,
	authority_discovery: AuthorityDiscoveryId,
	) -> SessionKeys {
	SessionKeys { babe, grandpa, im_online, authority_discovery }
}

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Generate an authority keys.
pub fn get_authority_keys_from_seed(seed: &str)
	-> (AccountId, AccountId, GrandpaId, BabeId, ImOnlineId, AuthorityDiscoveryId) {
	(
		get_account_id_from_seed::<sr25519::Public>(&format!("{}//stash", seed)),
		get_account_id_from_seed::<sr25519::Public>(seed),
		get_from_seed::<GrandpaId>(seed),
		get_from_seed::<BabeId>(seed),
		get_from_seed::<ImOnlineId>(seed),
		get_from_seed::<AuthorityDiscoveryId>(seed),
	)
}

pub fn development_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "WASM binary not available".to_string())?;
	Ok(ChainSpec::from_genesis(
		// Name
		"Development",
		// ID
		"dev",
		ChainType::Development,
		move || testnet_genesis(
			wasm_binary,
			// Initial PoA authorities
			vec![
				get_authority_keys_from_seed("Alice"),
			],
			// Sudo account
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			// Pre-funded accounts
			vec![
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				get_account_id_from_seed::<sr25519::Public>("Bob"),
				get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
				get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
			],
		),
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		// Properties
		Some(dust_properties()),
		// Extensions
		Default::default(),
	))
}

pub fn local_testnet_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "WASM binary not available".to_string())?;
	Ok(ChainSpec::from_genesis(
		// Name
		"Local Testnet",
		// ID
		"local_testnet",
		ChainType::Local,
		move || testnet_genesis(
			wasm_binary,
			// Initial PoA authorities
			vec![
				get_authority_keys_from_seed("Alice"),
				get_authority_keys_from_seed("Bob"),
			],
			// Sudo account
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			// Pre-funded accounts
			vec![
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				get_account_id_from_seed::<sr25519::Public>("Bob"),
				get_account_id_from_seed::<sr25519::Public>("Charlie"),
				get_account_id_from_seed::<sr25519::Public>("Dave"),
				get_account_id_from_seed::<sr25519::Public>("Eve"),
				get_account_id_from_seed::<sr25519::Public>("Ferdie"),
				get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
				get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
				get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
				get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
				get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
				get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
			],
		),
		// Bootnodes
		vec![],
		// Telemetry
		// TelemetryEndpoints::new(vec![(TELEMETRY_URL.into(), 0)]).ok(),
		None,
		// Protocol ID
		Some("dust_local_testnet"),
		// Properties
		Some(dust_properties()),
		// Extensions
		Default::default(),
	))
}

pub fn public_testnet_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "WASM binary not available".to_string())?;
	Ok(ChainSpec::from_genesis(
		// Name
		"Dust Testnet",
		// ID
		"dust_testnet",
		ChainType::Live,
		move || testnet_genesis(
			wasm_binary,
			// Initial authorities keys:
			// stash
			// controller
			// grandpa
			// babe
			// im-online
			// authority-discovery
			vec![
				(
					hex!["b2902b07056f7365bc22bf7e69c4e4fdba03e6af9c73ca6eb1703ccbc0248857"].into(),
					hex!["cc2ea454844cc1a2e821198d9e0ce1de1aee7d014af5dd3404fc8199df89f821"].into(),
					hex!["607712f6581e191b69046427a7e33c4713e96b4ae4654e2467c74279dc20beb2"].unchecked_into(),
					hex!["ba630d2df03743a6441ab9221a25fc00a62e6f3b56c6920634eebb72a15fc90f"].unchecked_into(),
					hex!["72c0d10c9cd6e44ccf5e7acf0bb1b7c4d6987dda55a36343f3d45b54ad8bfe32"].unchecked_into(),
					hex!["f287831caa53bc1dce6f0d676ab43d248921a4c34535be8f7d7d153eda29dc3f"].unchecked_into(),
				),
				(
					hex!["06ee8fc0e34e40f6f2c98328d70874c6dd7d7989159634c8c87301efbcbe4470"].into(),
					hex!["9cf9f939c16ef458e677472ff113af53e7fb9139244fcfa6fccb765aa8831019"].into(),
					hex!["db6d2cb33abebdc024a14ef7bfbc68823660be8d1acac66770e406e484de3184"].unchecked_into(),
					hex!["d09f879b3273d2cedab83fa741cdac328679c98914dc8dc07e359e19f0379844"].unchecked_into(),
					hex!["8c38deff9ab24a8c49e2b4fbdc963af7cbf06f99d6aabfaa6e50bfe6ae0d071d"].unchecked_into(),
					hex!["dcc1644697e98d4171a29074a4bfaeb49b39b6ea91a8ec5e049d23ea3c4a4134"].unchecked_into(),
				),
				(
					hex!["48267bffea5e524f1c0e06cce77f0ef920be7ed9a7dd47705e181edad64f532a"].into(),
					hex!["38594d7640612c49337f3a0bc7b39232b86f9c9c4fedec3f8b00e45d3f073a2d"].into(),
					hex!["c8996b17688cab9bcda8dafb4dde9bab4d9b1dc81c71419fca46fedcba74a14e"].unchecked_into(),
					hex!["568c17ce5ef308bd9544e7b16f34089a2c2329193f31577a830ffe8a023a6874"].unchecked_into(),
					hex!["66db4135f59db92ce98cdd6c29befaf21a93f1a9059adc2326c7d371a214f97d"].unchecked_into(),
					hex!["00858734321b53f0987a45906cbb91fe7ce1588fce03758c7c07f09022372c30"].unchecked_into(),
				),
			],
			// Sudo
			hex!["0c994e7589709a85128a6695254af16227f7873816ae0269aa705861c315ba1e"].into(),
			// Endowed accounts
			vec![
				hex!["0c994e7589709a85128a6695254af16227f7873816ae0269aa705861c315ba1e"].into(),
				hex!["9e42365c1a43fe7bd886118f49a2247aabda7079c3e4c5288f41afadd7bb1963"].into(),
				hex!["6c1371ce4b06b8d191d6f552d716c00da31aca08a291ccbdeaf0f7aeae51201b"].into(),
			],
		),
		// Bootnodes
		vec!["/dns/bootnode-t1.dustscan.com/tcp/30334/p2p/12D3KooWKmFtS7BFtkkKWrP5ZcCpPFokmST2JFXFSsVBNeW5SXWg".parse().unwrap()],
		// Telemetry
		TelemetryEndpoints::new(vec![(TELEMETRY_URL.into(), 0)]).ok(),
		// Protocol ID
		Some("dust_testnet"),
		// Properties
		Some(dust_properties()),
		// Extensions
		Default::default(),
	))
}


pub fn live_mainnet_config() -> Result<ChainSpec, String> {
	ChainSpec::from_json_bytes(&include_bytes!("../../assets/chain_spec_mainnet_raw.json")[..])
}

pub fn live_testnet_config() -> Result<ChainSpec, String> {
	ChainSpec::from_json_bytes(&include_bytes!("../../assets/chain_spec_testnet_raw.json")[..])
}

pub fn mainnet_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "WASM binary not available".to_string())?;
	Ok(ChainSpec::from_genesis(
		// Name
		"Dust Mainnet",
		// ID
		"dust_mainnet",
		ChainType::Live,
		move || mainnet_genesis(
			wasm_binary,
			// Initial authorities keys:
			// stash
			// controller
			// grandpa
			// babe
			// im-online
			// authority-discovery
			vec![
				(
					hex!["daa5961f930982e5253aa46ce78572b6a43cc754fa0ed4c34a088051ceffdc1e"].into(),
					hex!["aa753630bed0be36a4d96624bf96d7f50695c52cb5d86e09b49935fe3b66f02a"].into(),
					hex!["5636d0cd27e3f0ae9ed8940b449bd93786252c5076525ef0aae6f7e837ffc240"].unchecked_into(),
					hex!["d6815d35696c72625583e8ac7efc34a0ce8a9d189860076a8a7e4873ed2c850b"].unchecked_into(),
					hex!["6a13802d453e915729f3effafee1e52cb2ce9133c7da970efc47b23001049c0a"].unchecked_into(),
					hex!["0c895363573a60f5eb8f068476b1c49d2445b7c172f1918e9e04dc07d517de1f"].unchecked_into(),
				),
				(
					hex!["804bf8c7e697f48fef8ad06cdd8f473df122edf4290103976febcb109df29a04"].into(),
					hex!["68e704ecb8b6ead143ccc85668f38f8cc406b04ebf7227936284aa46d4923078"].into(),
					hex!["fc7473294f2919499b73776446ae6ddbb4b467a05f58695b81c43bf41e034323"].unchecked_into(),
					hex!["1c64f4cda26f97da46b18f8102c59f141a846c74659307881c80837bb92d2e41"].unchecked_into(),
					hex!["189b988669403999c4c9ba1cb76f9429d873f0d3956bbc869f1f36530bdedf61"].unchecked_into(),
					hex!["caf7fcc0a4f36c961e9de08fc22a1916c59177c8902f3f696f751c4278c66d54"].unchecked_into(),
				),
				(
					hex!["441f95d1a36b2eea7382bd326325efe2678bcacbeae34de740ffbb90b693c735"].into(),
					hex!["0ca26fbb87347b36feba5a25193cb9604821ee446f2321c7909f42b42d54b54e"].into(),
					hex!["0a8a5bb71d7288d135f38f6815aaa35b2690835e0e1ce3eaa431250b4a33323a"].unchecked_into(),
					hex!["ea29d36ccc943ad1605c68fe2524592cb3bef4b0b3e7721e6af69674fe058466"].unchecked_into(),
					hex!["9ee08f9ccf7d75c5a51db00e6b6c359857e23b0ddfbf32fab55ebf1348872d11"].unchecked_into(),
					hex!["36719e3625298854880ade3a3a19bd69384550967939b5988d8831b3a7b11128"].unchecked_into(),
				),
			],
			// Sudo
			hex!["bc30713c8c949b55557d600b0e9e3ed26e9eb60f031404f953281750c6ec4e2d"].into(),
			// Endowed accounts
			vec![
				// Investors
				(hex!["d8999d2094d6f7a7a5cc61932b51df2786c21d1c1058ec0eebbd98635c9a8e21"].into(), 4_340_893_656 as u128),
				// Liquidity bridge reserves
				(hex!["feecdaa17e46d038e885fa088bc9b07b060e11fcd70b1729dbeaf9294cdeca49"].into(), 2_000_000_000 as u128),
				// Lockup & core nominators
				(hex!["6e96d4d1c70847745aa40e07694867eb4eda164a1cc3be336439aaa416e4e642"].into(), 500_000_000 as u128),
				(hex!["dad98c159046c5be8ea8781863add146a24eda17f609bfa3075e3ec7f519d070"].into(), 500_000_000 as u128),
				(hex!["f0a7b25715e5cfc9b1573033a1959e0488a8739ed5e7cd56ad803abf135e687e"].into(), 500_000_000 as u128),
				(hex!["029ddde77f1a64af663e40494369feb06402df9c5b2b7bcb8547ecbb4424fe54"].into(), 250_000_000 as u128),
				(hex!["98f84017133351d6769c227a6dc1d504ffaa9de4b287ce37ce30fc944221fb74"].into(), 250_000_000 as u128),
				(hex!["1e291d5ba71592e71c5f1d32f3e8d4e96fd895c67291fa3836b36fcf5689e049"].into(), 250_000_000 as u128),
				// Sudo
				(hex!["bc30713c8c949b55557d600b0e9e3ed26e9eb60f031404f953281750c6ec4e2d"].into(), 100_000_000 as u128),
				// Developer pool & faucet
				(hex!["9468fc327624243168529b89eed8254b893364a3ae35fe2e31566ab9e4266e0b"].into(), 10_000_000 as u128),
			],
		),
		// Bootnodes
		vec![
			"/dns/bootnode.dustscan.com/tcp/30333/p2p/12D3KooWFHSc9cUcyNtavUkLg4VBAeBnYNgy713BnovUa9WNY5pp".parse().unwrap(),
			"/dns/bootnode.dust.llc/tcp/30333/p2p/12D3KooWAQqcXvcvt4eVEgogpDLAdGWgR5bY1drew44We6FfJAYq".parse().unwrap(),
			"/dns/bootnode.dust-chain.com/tcp/30333/p2p/12D3KooWCT7rnUmEK7anTp7svwr4GTs6k3XXnSjmgTcNvdzWzgWU".parse().unwrap(),
		],
		// Telemetry
		TelemetryEndpoints::new(vec![(TELEMETRY_URL.into(), 0)]).ok(),
		// Protocol ID
		Some("dust_mainnet"),
		// Properties
		Some(dust_properties()),
		// Extensions
		Default::default(),
	))
}

fn testnet_genesis(
	wasm_binary: &[u8],
	initial_authorities: Vec<(AccountId, AccountId, GrandpaId, BabeId, ImOnlineId, AuthorityDiscoveryId)>,
	root_key: AccountId,
	endowed_accounts: Vec<AccountId>,
) -> GenesisConfig {

	let evm_genesis_accounts = evm_genesis();

	const INITIAL_BALANCE: u128 = 100_000_000 * DUST;
	const INITIAL_STAKING: u128 =   1_000_000 * DUST;
	let existential_deposit = MaxNativeTokenExistentialDeposit::get();

	let balances = initial_authorities
		.iter()
		.map(|x| (x.0.clone(), INITIAL_STAKING))
		.chain(endowed_accounts.iter().cloned().map(|k| (k, INITIAL_BALANCE)))
		.chain(
			get_all_module_accounts()
				.iter()
				.map(|x| (x.clone(), existential_deposit)),
		)
		.fold(
			BTreeMap::<AccountId, Balance>::new(),
			|mut acc, (account_id, amount)| {
				if let Some(balance) = acc.get_mut(&account_id) {
					*balance = balance
						.checked_add(amount)
						.expect("balance cannot overflow when building genesis");
				} else {
					acc.insert(account_id.clone(), amount);
				}
				acc
			},
		)
		.into_iter()
		.collect::<Vec<(AccountId, Balance)>>();

	GenesisConfig {
		system: SystemConfig {
			// Add Wasm runtime to storage.
			code: wasm_binary.to_vec(),
			changes_trie_config: Default::default(),
		},
		indices: IndicesConfig { indices: vec![] },
		balances: BalancesConfig { balances },
		session: SessionConfig {
			keys: initial_authorities
				.iter()
				.map(|x| (
						x.0.clone(), // stash
						x.0.clone(), // stash
						get_session_keys(
							x.2.clone(), // grandpa
							x.3.clone(), // babe
							x.4.clone(), // im-online
							x.5.clone(), // authority-discovery
						)))
				.collect::<Vec<_>>(),
		},
		staking: StakingConfig {
			validator_count: initial_authorities.len() as u32 * 2,
			minimum_validator_count: initial_authorities.len() as u32,
			stakers: initial_authorities
				.iter()
				.map(|x| (x.0.clone(), x.1.clone(), INITIAL_STAKING, StakerStatus::Validator))
				.collect(),
			invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
			slash_reward_fraction: sp_runtime::Perbill::from_percent(10),
			..Default::default()
		},
		babe: BabeConfig { authorities: Default::default(), epoch_config: Some(BABE_GENESIS_EPOCH_CONFIG) },
		grandpa: Default::default(),
		authority_discovery: AuthorityDiscoveryConfig { keys: vec![] },
		im_online: Default::default(),
		tokens: TokensConfig {
			balances: endowed_accounts
				.iter()
				.flat_map(|x| {
					vec![
						(x.clone(), CurrencyId::Token(TokenSymbol::USDD), INITIAL_BALANCE),
					]
				})
				.collect(),
		},
		evm: EVMConfig {
			accounts: evm_genesis_accounts,
		},
		sudo: SudoConfig { key: root_key },
		tech_council: Default::default(),
	}
}

fn mainnet_genesis(
	wasm_binary: &[u8],
	initial_authorities: Vec<(AccountId, AccountId, GrandpaId, BabeId, ImOnlineId, AuthorityDiscoveryId)>,
	root_key: AccountId,
	endowed_accounts: Vec<(AccountId, Balance)>,
) -> GenesisConfig {

	let evm_genesis_accounts = evm_genesis();

	const INITIAL_STAKING: u128 = 1_000_000 * DUST;
	let existential_deposit = MaxNativeTokenExistentialDeposit::get();

	let balances = initial_authorities
		.iter()
		.map(|x| (x.0.clone(), INITIAL_STAKING*2))
		.chain(endowed_accounts.iter().cloned().map(|x| (x.0.clone(), x.1 * DUST)))
		.chain(
			get_all_module_accounts()
				.iter()
				.map(|x| (x.clone(), existential_deposit)),
		)
		.fold(
			BTreeMap::<AccountId, Balance>::new(),
			|mut acc, (account_id, amount)| {
				if let Some(balance) = acc.get_mut(&account_id) {
					*balance = balance
						.checked_add(amount)
						.expect("balance cannot overflow when building genesis");
				} else {
					acc.insert(account_id.clone(), amount);
				}
				acc
			},
		)
		.into_iter()
		.collect::<Vec<(AccountId, Balance)>>();

	GenesisConfig {
		system: SystemConfig {
			// Add Wasm runtime to storage.
			code: wasm_binary.to_vec(),
			changes_trie_config: Default::default(),
		},
		indices: IndicesConfig { indices: vec![] },
		balances: BalancesConfig { balances },
		session: SessionConfig {
			keys: initial_authorities
				.iter()
				.map(|x| (
						x.0.clone(), // stash
						x.0.clone(), // stash
						get_session_keys(
							x.2.clone(), // grandpa
							x.3.clone(), // babe
							x.4.clone(), // im-online
							x.5.clone(), // authority-discovery
						)))
				.collect::<Vec<_>>(),
		},
		staking: StakingConfig {
			validator_count: initial_authorities.len() as u32 * 2,
			minimum_validator_count: initial_authorities.len() as u32,
			stakers: initial_authorities
				.iter()
				.map(|x| (x.0.clone(), x.1.clone(), INITIAL_STAKING, StakerStatus::Validator))
				.collect(),
			invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
			slash_reward_fraction: sp_runtime::Perbill::from_percent(10),
			..Default::default()
		},
		babe: BabeConfig { authorities: Default::default(), epoch_config: Some(BABE_GENESIS_EPOCH_CONFIG) },
		grandpa: Default::default(),
		authority_discovery: AuthorityDiscoveryConfig { keys: vec![] },
		im_online: Default::default(),
		tokens: TokensConfig {
			balances: vec![]
		},
		evm: EVMConfig {
			accounts: evm_genesis_accounts,
		},
		sudo: SudoConfig { key: root_key },
		tech_council: Default::default(),
	}
}


/// Token
pub fn dust_properties() -> Properties {
	let mut p = Properties::new();
	p.insert("ss58format".into(), 42.into());
	p.insert("tokenDecimals".into(), 18.into());
	p.insert("tokenSymbol".into(), "DUST".into());
	p
}


/// Predeployed contract addresses
pub fn evm_genesis() -> BTreeMap<H160, module_evm::GenesisAccount<Balance, Nonce>> {
	let existential_deposit = MaxNativeTokenExistentialDeposit::get();
	let contracts_json = &include_bytes!("../../assets/bytecodes.json")[..];
	let contracts: Vec<(String, String, String)> = serde_json::from_slice(contracts_json).unwrap();
	let mut accounts = BTreeMap::new();
	for (_, address, code_string) in contracts {
		let account = module_evm::GenesisAccount {
			nonce: 0,
			balance: existential_deposit,
			storage: Default::default(),
			code: Bytes::from_str(&code_string).unwrap().0,
		};
		let addr = H160::from_slice(
			from_hex(address.as_str())
				.expect("predeploy-contracts must specify address")
				.as_slice(),
		);
		accounts.insert(addr, account);
	}
	accounts
}
