package org.scaleri.client.module.visual;

import net.minecraft.client.option.SimpleOption;
import org.scaleri.client.mixin.SimpleOptionAccessor;
import org.scaleri.client.module.Category;
import org.scaleri.client.module.Module;

public class FullbrightModule extends Module {
    private static final double BRIGHT = 15.0;
    private Double saved;

    public FullbrightModule() {
        super("Fullbright", "Lets you see clearly in caves and at night.", Category.VISUAL, false);
    }

    @SuppressWarnings("unchecked")
    private static SimpleOptionAccessor<Double> gamma() {
        return (SimpleOptionAccessor<Double>) (Object) mc.options.getGamma();
    }

    @Override
    public void onTick() {
        SimpleOption<Double> option = mc.options.getGamma();
        if (option.getValue() != BRIGHT) {
            if (saved == null) {
                saved = option.getValue();
            }
            gamma().scaleri$setValue(BRIGHT);
        }
    }

    @Override
    protected void onDisable() {
        restore();
    }

    /** Puts the real gamma back, e.g. before options.txt is written. */
    public void restore() {
        if (saved != null) {
            gamma().scaleri$setValue(saved);
            saved = null;
        }
    }
}
