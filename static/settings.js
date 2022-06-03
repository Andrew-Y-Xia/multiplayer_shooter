let SETTINGS;
await $.getJSON( "./settings.json", function( data ) {
    SETTINGS = data;
});

export function get_settings() {
    return SETTINGS;
}
