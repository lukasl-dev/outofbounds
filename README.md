# outofbounds

Automated inventory monitoring and Matrix notification system. `outofbounds` tracks your items in [Homebox](https://homebox.software/) and sends a notification to a Matrix room when quantities fall below a configured threshold.

## Usage

### NixOS Module

Add `outofbounds` to your flake inputs and use the provided module:

```nix
{
  inputs.outofbounds.url = "github:lukasl-dev/outofbounds";

  outputs = { nixpkgs, outofbounds, ... }: {
    nixosConfigurations.my-system = nixpkgs.lib.nixosSystem {
      modules = [
        outofbounds.nixosModules.default
        {
          services.outofbounds = {
            enable = true;
            interval = "hourly"; # systemd.time format
            
            # Option A: Attribute set (Nix will generate the TOML)
            settings = {
              homebox = {
                base_url = "https://homebox.example.com";
                username = "myuser";
                # Use file paths for secrets (sops-nix compatible)
                password_file = config.sops.secrets.homebox-pass.path;
                items = [
                  { id = "00000000-0000-0000-0000-000000000000"; threshold = 5; }
                ];
              };
              matrix = {
                user = "@bot:matrix.org";
                password_file = config.sops.secrets.matrix-pass.path;
                room_id = "!roomid:matrix.org";
                messages = [
                  {
                    plain = "⚠️ Low on {name}! Only {quantity} left.";
                    html = "⚠️ <b>Low on {name}!</b> Only <code>{quantity}</code> left.";
                  }
                ];
              };
            };

            # Option B: Path to a file (e.g. sops-nix template)
            # settings = config.sops.templates."outofbounds.toml".path;
          };
        }
      ];
    };
  };
}
```

### Manual Configuration

If not using NixOS, `outofbounds` looks for a `config.toml` in the current directory, or you can provide a path as the first argument:

```bash
outofbounds /path/to/my-config.toml
```

Example `config.toml`:

```toml
[matrix]
user = "@bot:example.com"
password = "my-secret-password" # or use password_file
room_id = "!abc:example.com"

[[matrix.messages]]
plain = "⚠️ Alarm! We only have {quantity} of {name} left."
html = "⚠️ <b>Emergency!</b> We only have <b>{quantity}</b> of <code>{name}</code> left."

[homebox]
base_url = "https://demo.homebox.software"
username = "admin"
password = "password"
items = [
    { id = "uuid-of-item", threshold = 2 }
]
```

## Placeholders

Templates support the following placeholders:
- `{name}`: Item name from Homebox.
- `{quantity}`: Current quantity.
- `{threshold}`: The configured limit.
- `{asset_id}`: Homebox asset ID.
- `{id}`: Homebox item UUID.

