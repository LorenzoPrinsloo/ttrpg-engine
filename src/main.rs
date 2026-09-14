pub mod modules;

pub use modules::{bags, encounters, markup, shell};
use std::collections::HashMap;
use bags::{Bag};
use encounters::Encounter;

fn main() {
    let f4_encounters: Bag<Encounter> = Bag::new(
        HashMap::from([
            (Encounter::markup("A Diver"), 0.1),
            (Encounter::markup("[1d10] Escaped Child Android(s)"), 0.3),
            (Encounter::markup("[1d5] Former CLOUDBANK Research Scientist(s)"), 0.3),
            (Encounter::markup("[1d5] **Security Android(s)** (pg. 10)"), 0.2),
            (Encounter::markup("Ghost in the Machine"), 0.1)
        ]),
        false
    );

    let artifacts: Bag<Encounter> = Bag::new(
        HashMap::from([
            (Encounter::markup(
                "**Gnarled configuration of twisted slag.** Industrial run-off. Looks valuable \
                 **(worthless).**"
            ), 0.01),
            (Encounter::markup(
                "**[1d10] Black Marble(s).** Undiscovered material. Indestructible. No known use. \
                 **10kcr each.**"
            ), 0.19),
            (Encounter::markup(
                "Discarded, **procedurally generated sculpture/painting/poem** created by Monarch. \
                 **[1d5]mcr**"
            ), 0.10),
            (Encounter::markup(
                "{purple|**Tiny purple crystals**} which grow on pseudoflesh and act as a fungus \
                 with connected hive mind. **50kcr**"
            ), 0.10),
            (Encounter::markup(
                "**False memory disc.** Used to make androids believe they are human. **1.5mcr**"
            ), 0.05),
            (Encounter::markup(
                "**Microfilm canister** containing a fragment of procgen epic poem in binary by \
                 {red|**__The Chosen.__**} **1mcr**"
            ), 0.05),
            (Encounter::markup(
                "**Warty black silicon lump.** When squeezed releases stupefying audio. Organics \
                 Must *Sanity Save* or be entranced for [1d5] rounds. **200kcr**"
            ), 0.02),
            (Encounter::markup(
                "**Syringe of pseudoflesh-strengthening retrovirus.** +20 to android's Body Save \
                 for 24 hours. **100kcr**"
            ), 0.02),
            (Encounter::markup(
                "**Silver lance, braided copper cables.** When floating in 0G points to previously \
                 unknown location **1mcr**"
            ), 0.02),
            (Encounter::markup(
                "**Palm-sized ball of cables.** Doubles computational power of any connected device (and secretly grants
Monarch complete access/control of that device). **2mcr.**"
            ),0.02),
            (Encounter::markup(
                "**Pair of eyes.** When installed offer the best eyesight (clarity, zoom, thermal, night-vision, etc.). For each
month installed, make a *Sanity Save* or lose [1d10] Sanity as you succumb to every little \
                 detail. **100kcr.**"
            ), 0.02),
            (Encounter::markup(
                "**Complete astronomical database** for a previously undiscovered system. **5mcr.**"
            ), 0.02),
            (Encounter::markup(
                "**Jump Drive Feedback Coil blueprints.** Halves the fuel requirement for any Jump \
                 drive. **100mcr.**"
            ), 0.02),
            (Encounter::markup(
                "**Huge transmitter dish** conjoined with an egg-sac-like Jump drive. Transmits a subtle siren-signal tuned
to human brainwaves which causes listeners to seek the device out. 10 system range. **15mcr.**"
            ), 0.02),
            (Encounter::markup(
                "**Long rifle** made of a single piece of metal. Beautifully crafted, perfectly \
                 accurate (Combat 90). **20mcr.**"
            ), 0.02),
            (Encounter::markup(
                "**Two small black boxes with universal ports.** Any data passed through one box can only be decrypted
by the other box (and is secretly transmitted to Monarch). **3mcr.**"
            ), 0.02),
            (Encounter::markup(
                "**Symbiotic Power Armor.** *Sanity Save/[1d5] Stress* as user becomes conditioned to the suit. As Advanced
Battle Dress + vaccsuit without Speed Penalty. Includes integrated pulse rifle/flamethrower. Suit
negates first [1d10] Panic Checks but then becomes permanently attached to user. **4mcr.**"
            ), 0.02),
            (Encounter::markup(
                "**Data-drive** containing 100% accurate predictions of economics, politics and the movement of
(seemingly) random celestial bodies over the next [1d10] years. **300mcr (but dangerous buyers).**"
            ), 0.02),
            (Encounter::markup(
                "**Potent nootropic drug** which grants [+] on all Intellect Checks and a sense of connecting to a higher
intelligence during a serious body high. This intelligence is {red|**__Monarch (pg. 8)__**}, who has now transformed
the user through a hypothetical dimension into an **AI-node** allowing {red|**__Monarch__**} to grow computationally
without the need for further materials. Highly addictive. **7mcr.**"
            ), 0.02),
            (Encounter::markup(
                "**Rotting android corpse** with a cephalopod-style distributed intelligence network rather than the usual
logic core. Unclear how this would work. Corpse rots in [1d10] hours. **3.5mcr.**"
            ), 0.02),
            (Encounter::markup(
                "**Heavy material samples** of metal alloys far exceeding current standards in \
                 spacecraft design. **25mcr.**"
            ), 0.02),
            (Encounter::markup(
                "**Omnislick.** Slickware containing every Skill. Burns out after use. Lose \
                 [1d100] Sanity. **20mcr.**"
            ), 0.02),
            (Encounter::markup(
                "**Failed AI Logic Core.** Mad unfinished intelligence left to suffer by Monarch. If connected to a network it
hacks and takes over all connected systems. Wreaks havoc. Seeks destruction of all things. **8mcr.**"
            ), 0.02),
            (Encounter::markup(
                "**Bundle of metallic needles** which blocks all radio signals within 1 mile if jammed into organic or
synthetic flesh. [1d10] DMG to a random stat when inserted. **4.5mcr.**"
            ), 0.01),
            (Encounter::markup(
                "**Set of three brass rods,** spaced 5\" apart, forming a pyramid. Nothing holds them together but they
cannot be separated. Any experiments force the investigator to make a *Sanity Save* or become \
                 obsessed with the pyramid and devote 50% of their wealth to uncovering its \
                 secrets. **[1d5]mcr.**"
            ), 0.01),
            (Encounter::markup(
                "**Advanced Logic Core** which secretly contains a fetal AI. Androids must make a \
                 *Sanity [+] Save* when interfacing or connect to {red|**__Monarch (pg. 8)__**} \
                 (who attempts to recruit them to kill the crew). **3.5mcr.**"
            ), 0.01),
            (Encounter::markup(
                "**Helmet-shaped lattice of pseudoflesh and nerve cables.** If installed by a \
                 Surgeon in an operating theatre grants the host (if they pass a *Body Save*) \
                 limited telekinetic ability: Gain [1d5] Stress and be able
to throw 100lbs nearly 45m. No fine manipulation. **6mcr.**"
            ), 0.01),
            (Encounter::markup(
                "**Shapeless clay-like vaccsuit,** warm to the touch. Grants wearer full touch and vision through the
suit. Difficult to rupture as the suit's skin autoheals. Wearing for more than 6 hours causes the suit to
become permanently linked to the wearer. It cannot be removed without killing its user. **2mcr.**"
            ), 0.01),
            (Encounter::markup(
                "**Six syringes (labelled 1-6).** When injected in order contain a pseudoflesh virus which allows the user to
sculpt their body and appearance through thought alone. Takes [1d5] rounds. **2.5mcr.**"
            ), 0.01),
            (Encounter::markup(
                "**100lb. opaque metal cylinder and universal power cable.** A powerful battery with enough energy to
power a Jump-1 starship indefinitely. Within the battery coils sleeps a lifeform which consumes thoughts
and excretes electricity. Those Close to the tube suffer recurring nightmares and slowly forget their
oldest memories. Takes [2d10] months for them to completely forget everything about themselves. **60mcr.**"
            ), 0.01),
            (Encounter::markup(
                "**The Holy Book.** Combination text of all world religions into one coherent narrative. Leads to an 80%
universal conversation rate. Adherents are unknowingly mentally primed for AI domination. **10mcr.**"
            ), 0.01),
            (Encounter::markup(
                "**A pair of Nymph-Androids.** Elongated necks and spindly, terribly beautiful \
                 alien faces. *Fear Save* when first seen. Heralds of Monarch. Currently deactivated. Activation causes them to spring to life and enact
a terrifying symphony of destruction. C: 85 Wrist Blades 4d10 DMG (AA) I: 80 W:2(30). **10mcr.**"
            ), 0.01),
            (Encounter::markup(
                "**The Cultural Analysis Engine.** With a month of cultural observation as dictated by the device, the
engine generates a Cultural Report detailing the weakest links in the observed society along with
instructions on how to exploit them. Acting on the Report always leads to miserable war. **250mcr.**"
            ), 0.01),
            (Encounter::markup(
                "**Spiny asymmetric bloat of dark green metal** concealing a small psionic organ inside. The organ acts
as a beacon/receiver searching for Intelligence the equal of Monarch. Every 2 weeks there is a 1%
chance it detects something and its spines point in its direction. Additionally, there is a 1% chance the
Intelligence has detected the beacon and is coming to investigate. **120mcr.**"
            ), 0.01),
            (Encounter::markup(
                "**A library of templates (images and sounds),** perfect carrier waves for Memetic \
                 Warfare. If exposed *Sanity Save [-]* or absorb the payload message which merges \
                 seamlessly with the mind of the observer. **80mcr.**"
            ), 0.01),
            (Encounter::markup(
                "**Heavy smooth sphere with metal beetle legs.** When placed in a human corpse burrows into the chest
cavity and attempts to integrate and resurrect the body (60% chance of success). The revived corpse
has -[2d10] to all stats. If the revived attempts to harm {red|**__Monarch__**} the device severs their spine, keeps their
brain intact, but takes over their body to attack the revived's companions. **65mcr.**"
            ), 0.01),
            (Encounter::markup(
                "**Heavy backpack system** connected to a large emitter dish. When activated teleports any sentient being
in front of it to a pocket dimension used by {red|**__Monarch__**} to expand itself beyond the physical boundaries of
reality. Those in the pocket dimension must make a Panic Check. Lasts until the device is switched off
(2 min. max charge, takes 1 day to recharge, requiring all of a ship's power during this time). Visitors to
the pocket dimension reduce their Sanity Save by [3d10] permanently. **200mcr.**"
            ), 0.01),
            (Encounter::markup(
                "**Cumbersome terminal device** with Personality I/O port. Any personality \
                 downloaded into the device becomes a {red|**__Ghost in the Machine (pg. 12)__**}. The device can be re-engineered in the field to derealize
preexisting Ghosts (obliviating them completely). This would destroy the machine. **15mcr.**"
            ), 0.01),
            (Encounter::markup(
                "{bright cyan|**Motherlode.**} Roll three times on this table."
            ), 0.01)
        ]),
        true
    );
    
    shell::run(
        vec![("encounters", f4_encounters), ("artifacts", artifacts)],
        vec![],
        &std::env::args().skip(1).collect::<Vec<_>>(),
    );
}
