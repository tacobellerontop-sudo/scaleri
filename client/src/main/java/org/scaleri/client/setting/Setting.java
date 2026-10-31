package org.scaleri.client.setting;

import com.google.gson.JsonElement;

/** A single user-tweakable value that belongs to a module. */
public abstract class Setting<T> {
    private final String name;
    private final T defaultValue;
    protected T value;

    protected Setting(String name, T defaultValue) {
        this.name = name;
        this.defaultValue = defaultValue;
        this.value = defaultValue;
    }

    public String getName() {
        return name;
    }

    /** Stable key used in the config file. */
    public String getId() {
        return name.toLowerCase().replaceAll("[^a-z0-9]+", "_");
    }

    public T get() {
        return value;
    }

    public void set(T value) {
        this.value = value;
    }

    public void reset() {
        set(defaultValue);
    }

    public abstract JsonElement toJson();

    public abstract void fromJson(JsonElement json);
}
