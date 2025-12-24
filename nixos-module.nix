{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.services.outofbounds;
  tomlFormat = pkgs.formats.toml { };
  mergedSettings = lib.recursiveUpdate cfg.settings {
    homebox = lib.optionalAttrs (cfg.homeboxPasswordFile != null) {
      password_file = toString cfg.homeboxPasswordFile;
    };
    matrix = lib.optionalAttrs (cfg.matrixPasswordFile != null) {
      password_file = toString cfg.matrixPasswordFile;
    };
  };
  configFile = tomlFormat.generate "outofbounds-config.toml" mergedSettings;
in
{
  options.services.outofbounds = {
    enable = lib.mkEnableOption "outofbounds";

    package = lib.mkOption {
      type = lib.types.package;
      description = "The outofbounds package to use.";
    };

    settings = lib.mkOption {
      type = tomlFormat.type;
      default = { };
      description = ''
        The configuration for outofbounds, as a Nix attribute set.
        See the project's config.rs for the expected structure.
      '';
    };

    homeboxPasswordFile = lib.mkOption {
      type = lib.types.nullOr lib.types.path;
      default = null;
      description = "Path to a file containing the Homebox password.";
    };

    matrixPasswordFile = lib.mkOption {
      type = lib.types.nullOr lib.types.path;
      default = null;
      description = "Path to a file containing the Matrix password.";
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
