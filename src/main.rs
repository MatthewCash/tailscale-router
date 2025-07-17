use anyhow::{Context, Result};
use futures_util::TryStreamExt;
use rtnetlink::{
    RouteMessageBuilder,
    packet_route::{
        route::{RouteProtocol, RouteScope},
        rule::RuleAction,
    },
};
use sd_notify::NotifyState;
use std::{env, net::Ipv4Addr, path::PathBuf};
use systemd::journal::{self};
use tracing_subscriber::{EnvFilter, filter::LevelFilter};

mod config;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .without_time() // systemd logs already include timestamps
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .with_env_var("LOG_LEVEL")
                .from_env()?,
        )
        .init();

    let config = config::load_config(PathBuf::from(
        env::var("CONFIG_PATH").expect("CONFIG_PATH env var missing!"),
    ))
    .await
    .context("failed to load config")?;

    let (netlink_conn, netlink_handle, _) = rtnetlink::new_connection()?;
    tokio::spawn(netlink_conn);

    log::debug!("connected to netlink");

    let mut reader = journal::OpenOptions::default()
        .open()
        .context("Could not open journal")?;

    log::debug!("opened journald logs");

    reader
        .seek_tail()
        .context("failed to seek to end of journald logs")?;
    reader
        .previous()
        .context("failed to step back in journal")?;

    netlink_handle
        .rule()
        .add()
        .v4()
        .fw_mark(config.vpn_route_fwmark)
        .table_id(config.vpn_route_table_id)
        .action(RuleAction::ToTable)
        .execute()
        .await
        .context("failed to add rule")?;

    let _ = sd_notify::notify(false, &[NotifyState::Ready]);
    log::info!("Waiting for journald entries...");

    loop {
        loop {
            let Some(entry) = reader
                .next_entry()
                .context("failed to get next journal entry")?
            else {
                break;
            };

            log::trace!("Received journal entry: {entry:?}");

            let message = entry
                .get("MESSAGE")
                .context("failed to get message from journal entry")?;

            log::trace!("Received journal entry with message: {message}");

            if entry
                .get("_SYSTEMD_UNIT")
                .is_none_or(|id| *id != config.tailscale_unit_id)
                || !message.contains(&config.journal_online_str)
            {
                continue;
            }

            log::info!("Triggering route update due to online log entry");

            let tailscale_iface_id = netlink_handle
                .link()
                .get()
                .match_name(config.tailscale_interface_name.clone())
                .execute()
                .try_next()
                .await?
                .context("failed to get tailscale interface id")?
                .header
                .index;

            log::debug!("got tailscale interface id: {tailscale_iface_id}");

            netlink_handle
                .route()
                .add(
                    RouteMessageBuilder::<Ipv4Addr>::new()
                        .destination_prefix(Ipv4Addr::new(0, 0, 0, 0), 0)
                        .output_interface(tailscale_iface_id)
                        .table_id(config.vpn_route_table_id)
                        .scope(RouteScope::Link)
                        .protocol(RouteProtocol::Boot)
                        .build(),
                )
                .execute()
                .await
                .context("failed to add route")?;

            netlink_handle
                .route()
                .del(
                    RouteMessageBuilder::<Ipv4Addr>::new()
                        .destination_prefix(Ipv4Addr::new(0, 0, 0, 0), 0)
                        .output_interface(tailscale_iface_id)
                        .table_id(config.tailscale_route_table_id)
                        .scope(RouteScope::Universe)
                        .protocol(RouteProtocol::Boot)
                        .build(),
                )
                .execute()
                .await
                .context("failed to delete tailscale route")?;
        }

        reader
            .wait(None)
            .context("failed to wait for journal entry")?;
    }
}
