package org.scaleri.client.setting;

import com.google.gson.JsonElement;
import com.google.gson.JsonPrimitive;
import java.util.List;

/** Picks one option out of a fixed list of strings. */
public class ModeSetting extends Setting<String> {
    private final List<String> modes;

    public ModeSetting(String name, String defaultValue, String... modes) {
        super(name, defaultValue);
        this.modes = List.of(modes);
    }

    public List<String> getModes() {
        return modes;
    }

    public boolean is(String mode) {
        return value.equals(mode);
    }

    public void cycle(boolean forward) {
        int i = modes.indexOf(value);
        int n = modes.size();
        set(modes.get(((i + (forward ? 1 : -1)) % n + n) % n));
    }

    @Override
    public JsonElement toJson() {
        return new JsonPrimitive(value);
    }

    @Override
    public void fromJson(JsonElement json) {
        if (json.isJsonPrimitive() && modes.contains(json.getAsString())) {
            set(json.getAsString());
        }
    }
}
