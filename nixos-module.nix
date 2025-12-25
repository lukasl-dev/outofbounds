{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.services.outofbounds;
  tomlFormat = pkgs.formats.toml { };
  configFile =
    if lib.isType "path" cfg.settings || lib.isPath cfg.settings || (lib.isString cfg.settings && lib.hasPrefix "/" cfg.settings) then
      cfg.settings
    else
      tomlFormat.generate "outofbounds-config.toml" cfg.settings;
in
{
  options.services.outofbounds = {
    enable = lib.mkEnableOption "outofbounds";

    package = lib.mkOption {
      type = lib.types.package;
      description = "The outofbounds package to use.";
    };

    settings = lib.mkOption {
      type = lib.types.either tomlFormat.type lib.types.path;
      default = { };
      description = ''
        The configuration for outofbounds, either as a Nix attribute set or a path to a TOML file.
        If a path is provided, it will be used directly (useful for sops-nix templates).
      '';
    };

    interval = lib.mkOption {
      type = lib.types.str;
      default = "hourly";
      description = "The interval at which to run outofbounds (systemd.time format).";
    };
  };

  config = lib.mkIf cfg.enable {
    systemd.services.outofbounds = {
      description = "outofbounds inventory notifier";
      serviceConfig = {
        Type = "oneshot";
        ExecStart = "${cfg.package}/bin/outofbounds ${configFile}";
        DynamicUser = true;
      };
    };

    systemd.timers.outofbounds = {
      description = "outofbounds inventory notifier timer";
      wantedBy = [ "timers.target" ];
      timerConfig = {
        OnCalendar = cfg.interval;
        Persistent = true;
      };
    };
  };
}
