// src/data.rs
// This module contains large static data tables for the materials library.
// NATURAL_ABUNDANCE: canonical natural     m for all stable isotopes.

use std::collections::HashMap;
use once_cell::sync::Lazy;

pub static NATURAL_ABUNDANCE: Lazy<HashMap<&'static str, f64>> = Lazy::new(|| {
    let mut m = HashMap::new();
    // ...existing NATURAL_ABUNDANCE data from element.rs...
    // Example entries (replace with full data):
    m.insert("Li6", 0.0759);
    m.insert("Li7", 0.9241);

    // Hydrogen
    m.insert("H1", 0.99984426);
    m.insert("H2", 0.00015574);
    
    // Helium
    m.insert("He3", 0.000002);
    m.insert("He4", 0.999998);
    
    // Lithium
    m.insert("Li6", 0.07589);
    m.insert("Li7", 0.92411);
    
    // Beryllium
    m.insert("Be9", 1.0);
    
    // Boron
    m.insert("B10", 0.1982);
    m.insert("B11", 0.8018);
    
    // Carbon
    m.insert("C12", 0.988922);
    m.insert("C13", 0.011078);
    
    // Nitrogen
    m.insert("N14", 0.996337);
    m.insert("N15", 0.003663);
    
    // Oxygen
    m.insert("O16", 0.9976206);
    m.insert("O17", 0.000379);
    m.insert("O18", 0.0020004);
    
    // Fluorine
    m.insert("F19", 1.0);
    
    // Neon
    m.insert("Ne20", 0.9048);
    m.insert("Ne21", 0.0027);
    m.insert("Ne22", 0.0925);
    
    // Sodium
    m.insert("Na23", 1.0);
    
    // Magnesium
    m.insert("Mg24", 0.78951);
    m.insert("Mg25", 0.1002);
    m.insert("Mg26", 0.11029);
    
    // Aluminum
    m.insert("Al27", 1.0);
    
    // Silicon
    m.insert("Si28", 0.9222968);
    m.insert("Si29", 0.0468316);
    m.insert("Si30", 0.0308716);
    
    // Phosphorus
    m.insert("P31", 1.0);
    
    // Sulfur
    m.insert("S32", 0.9504074);
    m.insert("S33", 0.0074869);
    m.insert("S34", 0.0419599);
    m.insert("S36", 0.0001458);

    // Chlorine
    m.insert("Cl35", 0.757647);
    m.insert("Cl37", 0.242353);

    // Argon
    m.insert("Ar36", 0.003336);
    m.insert("Ar38", 0.000629);
    m.insert("Ar40", 0.996035);

    // Potassium
    m.insert("K39", 0.932581);
    m.insert("K40", 0.000117);
    m.insert("K41", 0.067302);

    // Calcium
    m.insert("Ca40", 0.96941);
    m.insert("Ca42", 0.00647);
    m.insert("Ca43", 0.00135);
    m.insert("Ca44", 0.02086);
    m.insert("Ca46", 0.00004);
    m.insert("Ca48", 0.00187);

    // Scandium
    m.insert("Sc45", 1.0);

    // Titanium
    m.insert("Ti46", 0.0825);
    m.insert("Ti47", 0.0744);
    m.insert("Ti48", 0.7372);
    m.insert("Ti49", 0.0541);
    m.insert("Ti50", 0.0518);

    // Vanadium
    m.insert("V50", 0.0025);
    m.insert("V51", 0.9975);

    // Chromium
    m.insert("Cr50", 0.04345);
    m.insert("Cr52", 0.83789);
    m.insert("Cr53", 0.09501);
    m.insert("Cr54", 0.02365);

    // Manganese
    m.insert("Mn55", 1.0);
    
    // Iron
    m.insert("Fe54", 0.05845);
    m.insert("Fe56", 0.91754);
    m.insert("Fe57", 0.02119);
    m.insert("Fe58", 0.00282);

    // Cobalt
    m.insert("Co59", 1.0);

    // Nickel
    m.insert("Ni58", 0.680769);
    m.insert("Ni60", 0.262231);
    m.insert("Ni61", 0.011399);
    m.insert("Ni62", 0.036345);
    m.insert("Ni64", 0.009256);

    // Copper
    m.insert("Cu63", 0.6915);
    m.insert("Cu65", 0.3085);

    // Zinc
    m.insert("Zn64", 0.4917);
    m.insert("Zn66", 0.2773);
    m.insert("Zn67", 0.0404);
    m.insert("Zn68", 0.1845);
    m.insert("Zn70", 0.0061);

    // Gallium
    m.insert("Ga69", 0.60108);
    m.insert("Ga71", 0.39892);

    // Germanium
    m.insert("Ge70", 0.2052);
    m.insert("Ge72", 0.2745);
    m.insert("Ge73", 0.0776);
    m.insert("Ge74", 0.3652);
    m.insert("Ge76", 0.0775);

    // Arsenic
    m.insert("As75", 1.0);

    // Selenium
    m.insert("Se74", 0.0086);
    m.insert("Se76", 0.0923);
    m.insert("Se77", 0.076);
    m.insert("Se78", 0.2369);
    m.insert("Se80", 0.498);
    m.insert("Se82", 0.0882);

    // Bromine
    m.insert("Br79", 0.50686);
    m.insert("Br81", 0.49314);

    // Krypton
    m.insert("Kr78", 0.00355);
    m.insert("Kr80", 0.02286);
    m.insert("Kr82", 0.11593);
    m.insert("Kr83", 0.115);
    m.insert("Kr84", 0.56987);
    m.insert("Kr86", 0.17279);

    // Rubidium
    m.insert("Rb85", 0.7217);
    m.insert("Rb87", 0.2783);

    // Strontium
    m.insert("Sr84", 0.0056);
    m.insert("Sr86", 0.0986);
    m.insert("Sr87", 0.07);
    m.insert("Sr88", 0.8258);

    // Yttrium
    m.insert("Y89", 1.0);

    // Zirconium
    m.insert("Zr90", 0.5145);
    m.insert("Zr91", 0.1122);
    m.insert("Zr92", 0.1715);
    m.insert("Zr94", 0.1738);
    m.insert("Zr96", 0.028);

    // Niobium
    m.insert("Nb93", 1.0);

    // Molybdenum
    m.insert("Mo92", 0.14649);
    m.insert("Mo94", 0.09187);
    m.insert("Mo95", 0.15873);
    m.insert("Mo96", 0.16673);
    m.insert("Mo97", 0.09582);
    m.insert("Mo98", 0.24292);
    m.insert("Mo100", 0.09744);
    
    // Ruthenium
    m.insert("Ru96", 0.0554);
    m.insert("Ru98", 0.0187);
    m.insert("Ru99", 0.1276);
    m.insert("Ru100", 0.126);
    m.insert("Ru101", 0.1706);
    m.insert("Ru102", 0.3155);
    m.insert("Ru104", 0.1862);

    // Rhodium
    m.insert("Rh103", 1.0);

    // Palladium
    m.insert("Pd102", 0.0102);
    m.insert("Pd104", 0.1114);
    m.insert("Pd105", 0.2233);
    m.insert("Pd106", 0.2733);
    m.insert("Pd108", 0.2646);
    m.insert("Pd110", 0.1172);

    // Silver
    m.insert("Ag107", 0.51839);
    m.insert("Ag109", 0.48161);

    // Cadmium
    m.insert("Cd106", 0.01245);
    m.insert("Cd108", 0.00888);
    m.insert("Cd110", 0.1247);
    m.insert("Cd111", 0.12795);
    m.insert("Cd112", 0.24109);
    m.insert("Cd113", 0.12227);
    m.insert("Cd114", 0.28754);
    m.insert("Cd116", 0.07512);

    // Indium
    m.insert("In113", 0.04281);
    m.insert("In115", 0.95719);

    // Tin
    m.insert("Sn112", 0.0097);
    m.insert("Sn114", 0.0066);
    m.insert("Sn115", 0.0034);
    m.insert("Sn116", 0.1454);
    m.insert("Sn117", 0.0768);
    m.insert("Sn118", 0.2422);
    m.insert("Sn119", 0.0859);
    m.insert("Sn120", 0.3258);
    m.insert("Sn122", 0.0463);
    m.insert("Sn124", 0.0579);

    // Antimony
    m.insert("Sb121", 0.5721);
    m.insert("Sb123", 0.4279);

    // Tellurium
    m.insert("Te120", 0.0009);
    m.insert("Te122", 0.0255);
    m.insert("Te123", 0.0089);
    m.insert("Te124", 0.0474);
    m.insert("Te125", 0.0707);
    m.insert("Te126", 0.1884);
    m.insert("Te128", 0.3174);
    m.insert("Te130", 0.3408);

    // Iodine
    m.insert("I127", 1.0);

    // Xenon
    m.insert("Xe124", 0.00095);
    m.insert("Xe126", 0.00089);
    m.insert("Xe128", 0.0191);
    m.insert("Xe129", 0.26401);
    m.insert("Xe130", 0.04071);
    m.insert("Xe131", 0.21232);
    m.insert("Xe132", 0.26909);
    m.insert("Xe134", 0.10436);
    m.insert("Xe136", 0.08857);

    // Cesium
    m.insert("Cs133", 1.0);

    // Barium
    m.insert("Ba130", 0.0011);
    m.insert("Ba132", 0.001);
    m.insert("Ba134", 0.0242);
    m.insert("Ba135", 0.0659);
    m.insert("Ba136", 0.0785);
    m.insert("Ba137", 0.1123);
    m.insert("Ba138", 0.717);

    // Lanthanum
    m.insert("La138", 0.0008881);
    m.insert("La139", 0.9991119);

    // Cerium
    m.insert("Ce136", 0.00186);
    m.insert("Ce138", 0.00251);
    m.insert("Ce140", 0.88449);
    m.insert("Ce142", 0.11114);

    // Praseodymium
    m.insert("Pr141", 1.0);

    // Neodymium
    m.insert("Nd142", 0.27153);
    m.insert("Nd143", 0.12173);
    m.insert("Nd144", 0.23798);
    m.insert("Nd145", 0.08293);
    m.insert("Nd146", 0.17189);
    m.insert("Nd148", 0.05756);
    m.insert("Nd150", 0.05638);

    // Samarium
    m.insert("Sm144", 0.0308);
    m.insert("Sm147", 0.15);
    m.insert("Sm148", 0.1125);
    m.insert("Sm149", 0.1382);
    m.insert("Sm150", 0.0737);
    m.insert("Sm152", 0.2674);
    m.insert("Sm154", 0.2274);

    // Europium
    m.insert("Eu151", 0.4781);
    m.insert("Eu153", 0.5219);

    // Gadolinium
    m.insert("Gd152", 0.002);
    m.insert("Gd154", 0.0218);
    m.insert("Gd155", 0.148);
    m.insert("Gd156", 0.2047);
    m.insert("Gd157", 0.1565);
    m.insert("Gd158", 0.2484);
    m.insert("Gd160", 0.2186);

    // Terbium
    m.insert("Tb159", 1.0);

    // Dysprosium
    m.insert("Dy156", 0.00056);
    m.insert("Dy158", 0.00095);
    m.insert("Dy160", 0.02329);
    m.insert("Dy161", 0.18889);
    m.insert("Dy162", 0.25475);
    m.insert("Dy163", 0.24896);
    m.insert("Dy164", 0.2826);

    // Holmium
    m.insert("Ho165", 1.0);

    // Erbium
    m.insert("Er162", 0.00139);
    m.insert("Er164", 0.01601);
    m.insert("Er166", 0.33503);
    m.insert("Er167", 0.22869);
    m.insert("Er168", 0.26978);
    m.insert("Er170", 0.1491);

    // Thulium
    m.insert("Tm169", 1.0);

    // Ytterbium
    m.insert("Yb168", 0.00123);
    m.insert("Yb170", 0.02982);
    m.insert("Yb171", 0.14086);
    m.insert("Yb172", 0.21686);
    m.insert("Yb173", 0.16103);
    m.insert("Yb174", 0.32025);
    m.insert("Yb176", 0.12995);

    // Lutetium
    m.insert("Lu175", 0.97401);
    m.insert("Lu176", 0.02599);
    
    // Hafnium
    m.insert("Hf174", 0.0016);
    m.insert("Hf176", 0.0526);
    m.insert("Hf177", 0.186);
    m.insert("Hf178", 0.2728);
    m.insert("Hf179", 0.1362);
    m.insert("Hf180", 0.3508);

    // Tantalum
    m.insert("Ta180_m1", 0.0001201);
    m.insert("Ta181", 0.9998799);

    // Tungsten
    m.insert("W180", 0.0012);
    m.insert("W182", 0.265);
    m.insert("W183", 0.1431);
    m.insert("W184", 0.3064);
    m.insert("W186", 0.2843);

    // Rhenium
    m.insert("Re185", 0.374);
    m.insert("Re187", 0.626);

    // Osmium
    m.insert("Os184", 0.0002);
    m.insert("Os186", 0.0159);
    m.insert("Os187", 0.0196);
    m.insert("Os188", 0.1324);
    m.insert("Os189", 0.1615);
    m.insert("Os190", 0.2626);
    m.insert("Os192", 0.4078);

    // Iridium
    m.insert("Ir191", 0.373);
    m.insert("Ir193", 0.627);

    // Platinum
    m.insert("Pt190", 0.00012);
    m.insert("Pt192", 0.00782);
    m.insert("Pt194", 0.32864);
    m.insert("Pt195", 0.33775);
    m.insert("Pt196", 0.25211);
    m.insert("Pt198", 0.07356);

    // Gold
    m.insert("Au197", 1.0);

    // Mercury
    m.insert("Hg196", 0.0015);
    m.insert("Hg198", 0.1004);
    m.insert("Hg199", 0.1694);
    m.insert("Hg200", 0.2314);
    m.insert("Hg201", 0.1317);
    m.insert("Hg202", 0.2974);
    m.insert("Hg204", 0.0682);

    // Thallium
    m.insert("Tl203", 0.29524);
    m.insert("Tl205", 0.70476);

    // Lead
    m.insert("Pb204", 0.014);
    m.insert("Pb206", 0.241);
    m.insert("Pb207", 0.221);
    m.insert("Pb208", 0.524);

    // Bismuth
    m.insert("Bi209", 1.0);

    // Thorium
    m.insert("Th230", 0.0002);
    m.insert("Th232", 0.9998);

    // Protactinium
    m.insert("Pa231", 1.0);

    // Uranium
    m.insert("U234", 0.000054);
    m.insert("U235", 0.007204);
    m.insert("U238", 0.992742);
    m
});

pub static ELEMENT_NAMES: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut names = HashMap::new();
    names.insert("H", "hydrogen");
    names.insert("He", "helium");
    names.insert("Li", "lithium");
    names.insert("Be", "beryllium");
    names.insert("B", "boron");
    names.insert("C", "carbon");
    names.insert("N", "nitrogen");
    names.insert("O", "oxygen");
    names.insert("F", "fluorine");
    names.insert("Ne", "neon");
    names.insert("Na", "sodium");
    names.insert("Mg", "magnesium");
    names.insert("Al", "aluminum");
    names.insert("Si", "silicon");
    names.insert("P", "phosphorus");
    names.insert("S", "sulfur");
    names.insert("Cl", "chlorine");
    names.insert("Ar", "argon");
    names.insert("K", "potassium");
    names.insert("Ca", "calcium");
    names.insert("Sc", "scandium");
    names.insert("Ti", "titanium");
    names.insert("V", "vanadium");
    names.insert("Cr", "chromium");
    names.insert("Mn", "manganese");
    names.insert("Fe", "iron");
    names.insert("Co", "cobalt");
    names.insert("Ni", "nickel");
    names.insert("Cu", "copper");
    names.insert("Zn", "zinc");
    names.insert("Ga", "gallium");
    names.insert("Ge", "germanium");
    names.insert("As", "arsenic");
    names.insert("Se", "selenium");
    names.insert("Br", "bromine");
    names.insert("Kr", "krypton");
    names.insert("Rb", "rubidium");
    names.insert("Sr", "strontium");
    names.insert("Y", "yttrium");
    names.insert("Zr", "zirconium");
    names.insert("Nb", "niobium");
    names.insert("Mo", "molybdenum");
    names.insert("Tc", "technetium");
    names.insert("Ru", "ruthenium");
    names.insert("Rh", "rhodium");
    names.insert("Pd", "palladium");
    names.insert("Ag", "silver");
    names.insert("Cd", "cadmium");
    names.insert("In", "indium");
    names.insert("Sn", "tin");
    names.insert("Sb", "antimony");
    names.insert("Te", "tellurium");
    names.insert("I", "iodine");
    names.insert("Xe", "xenon");
    names.insert("Cs", "cesium");
    names.insert("Ba", "barium");
    names.insert("La", "lanthanum");
    names.insert("Ce", "cerium");
    names.insert("Pr", "praseodymium");
    names.insert("Nd", "neodymium");
    names.insert("Pm", "promethium");
    names.insert("Sm", "samarium");
    names.insert("Eu", "europium");
    names.insert("Gd", "gadolinium");
    names.insert("Tb", "terbium");
    names.insert("Dy", "dysprosium");
    names.insert("Ho", "holmium");
    names.insert("Er", "erbium");
    names.insert("Tm", "thulium");
    names.insert("Yb", "ytterbium");
    names.insert("Lu", "lutetium");
    names.insert("Hf", "hafnium");
    names.insert("Ta", "tantalum");
    names.insert("W", "tungsten");
    names.insert("Re", "rhenium");
    names.insert("Os", "osmium");
    names.insert("Ir", "iridium");
    names.insert("Pt", "platinum");
    names.insert("Au", "gold");
    names.insert("Hg", "mercury");
    names.insert("Tl", "thallium");
    names.insert("Pb", "lead");
    names.insert("Bi", "bismuth");
    names.insert("Po", "polonium");
    names.insert("At", "astatine");
    names.insert("Rn", "radon");
    names.insert("Fr", "francium");
    names.insert("Ra", "radium");
    names.insert("Ac", "actinium");
    names.insert("Th", "thorium");
    names.insert("Pa", "protactinium");
    names.insert("U", "uranium");
    names.insert("Np", "neptunium");
    names.insert("Pu", "plutonium");
    names.insert("Am", "americium");
    names.insert("Cm", "curium");
    names.insert("Bk", "berkelium");
    names.insert("Cf", "californium");
    names.insert("Es", "einsteinium");
    names.insert("Fm", "fermium");
    names.insert("Md", "mendelevium");
    names.insert("No", "nobelium");
    names.insert("Lr", "lawrencium");
    names
});
