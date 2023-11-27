# isotopic_icp_store

This is the backend canister for the Isotopic Game Store on ICP, providing utilities to upload, manage, and download game build files on the Internet Computer.

The [Isotopic Game Store](https://isotopic.io/game-store) connects with the live canister, allowing users to download game files that are stored on-chain, directly from the Isotopic website.
Developers that upload games to the store get to choose whether they want their build to be backed up and stored on the Internet Computer, for true decentralization of their game's distribution.

Canister's Capabilities:
- Upload build files for games in chunks.
- Download chunks of build files by given ``upload ID``.
- Retrieve assigned builds per platform for a given ``Isotopic App ID``.
- Assign uploads to an app id, delete uploads. 

## About Isotopic

Isotopic is a new Open and Cross-Platform Game Store, where games become ownable assets that can be traded, lended, resold, or otherwise repurposed. Browse, buy, download, and play unlimited games and applications for desktop, mobile, web, and VR environments.

Links:
- Website: [link](https://isotopic.io)
- Live Game Store: [link](https://isotopic.io/game-store)
- Socials/Communities: [Twitter](https://twitter.com/isotopic12) | [Discord](https://discord.gg/zZqNycn6FJ) | [Telegram](https://t.me/+agHbqwIuW95jMzdk)

![Isotopic Banner](https://dapp.isotopic.io/media/isotopic-banner-text.jpg)

# For Developers

The project is written in Rust with ICP's [Rust Canister Development Kit](https://github.com/dfinity/cdk-rs).

To learn more before you start working with isotopic_icp_store, see the following documentation available online regarding the Internet Computer network:

- [Quick Start](https://internetcomputer.org/docs/current/developer-docs/setup/deploy-locally)
- [SDK Developer Tools](https://internetcomputer.org/docs/current/developer-docs/setup/install)
- [Rust Canister Development Guide](https://internetcomputer.org/docs/current/developer-docs/backend/rust/)
- [ic-cdk](https://docs.rs/ic-cdk)
- [ic-cdk-macros](https://docs.rs/ic-cdk-macros)
- [Candid Introduction](https://internetcomputer.org/docs/current/developer-docs/backend/candid/)


If you want to start working on your project right away, you might want to try the following commands:

```bash
cd isotopic_icp_store/
dfx help
dfx canister --help
```


## Running the project locally

If you want to test your project locally, you can use the following commands:

```bash
# Starts the replica, running in the background
dfx start --background

# Deploys your canisters to the replica and generates your candid interface, giving authorization to the provided principals
dfx deploy isotopic_icp_store_backend --argument '(record { owners = opt vec {principal "<principal 1>"; principal "<principal 2>"};})'
```

Once the job completes, your application will be available at `http://localhost:4943?canisterId={asset_canister_id}`.

If you have made changes to your backend canister, you can generate a new candid interface with

```bash
npm run generate
```

at any time. This is recommended before starting the frontend development server, and will be run automatically any time you run `dfx deploy`.
