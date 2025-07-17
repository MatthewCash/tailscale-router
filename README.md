# tailscale-router

Applies routing changes after tailscale vpn is connected.

- Disables routing `0.0.0.0/0` through tailscale, allowing tailscale to think it is running as an exit node while not acting as one.

- Adds a rule for packets with a configurable fwmark to be routed through tailscale.

## Configuration

Example:

```toml
tailscale_unit_id = "tailscaled.service"
tailscale_interface_name = "tailscale0"
tailscale_route_table_id = 52
journal_online_str = "Switching ipn state Starting -> Running"
vpn_route_table_id = 100
vpn_route_fwmark = 0x2
```

## Running

Simply execute:

`CONFIG_PATH=config.toml cargo run`

Or use the provided systemd service:

```bash
cargo build --release
sudo install -m 755 ./target/release/tailscale-router /usr/local/bin/
sudo install -m 644 tailscale-router.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable --now tailscale-router
```
