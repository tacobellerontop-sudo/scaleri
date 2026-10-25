package org.scaleri.client.module.pvp;

import net.minecraft.entity.Entity;
import net.minecraft.entity.LivingEntity;
import net.minecraft.particle.ParticleTypes;
import org.scaleri.client.module.Category;
import org.scaleri.client.module.Module;
import org.scaleri.client.setting.BoolSetting;
import org.scaleri.client.setting.NumberSetting;

public class HitParticlesModule extends Module {
    private final BoolSetting alwaysCrit = add(new BoolSetting("Always Show Crits", true));
    private final BoolSetting alwaysSharpness = add(new BoolSetting("Always Show Sharpness", false));
    private final NumberSetting multiplier = add(new NumberSetting("Multiplier", 2, 1, 5, 1));

    public HitParticlesModule() {
        super("Hit Particles", "Adds extra crit and sharpness particles to your hits.", Category.PVP, false);
    }

    public void onAttack(Entity target) {
        if (!(target instanceof LivingEntity) || mc.particleManager == null) {
            return;
        }
        for (int i = 0; i < multiplier.getInt(); i++) {
            if (alwaysCrit.isOn()) {
                mc.particleManager.addEmitter(target, ParticleTypes.CRIT);
            }
            if (alwaysSharpness.isOn()) {
                mc.particleManager.addEmitter(target, ParticleTypes.ENCHANTED_HIT);
            }
        }
    }
}
