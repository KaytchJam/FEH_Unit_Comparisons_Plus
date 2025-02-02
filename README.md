An updated version of the FEH Unit Comparison, the repository for that can be found [here](https://github.com/KaytchJam/FEH_Unit_Comparisons):

Dataset is courtesy of u/_vinventure and the many other contributors to the FEH Unit spreadsheet [here](https://www.reddit.com/r/FireEmblemHeroes/comments/12vgm0u/introducing_a_filterable_spreadsheet_of_all_feh/).

![Unit Interpolation Figure 1](assets/unit_interpolation_figure_1.png)

The figure above is a visualization of what this repository does. We take two FEH Units, construct an interpolating line between said units (treated as points based on their stats), and then find the nearest unit to said interpolation point based on a given metric. Nearest neighbor computation is sped up through the use of a KDTree data structure. 

![Interpolating between Lon'Qu and Winter Edelgard. The nearest units we pass by are: Chad, Nils, Echida, Valentine's Eliwood, Halloween Kurthnaga, Ascended Eliwood, Halloween Corrin, and Summer Edelgard.](./examples/lonqu_to_winter_edelgard.gif "Lon'Qu to Winter Edelgard")

![Interpolating between Camus and Sirius. We only pass by one unit, being Seigbert.](./examples/camus_to_sirius.gif "Camus to Sirius")

![Interpolating between Ike and Roy. We pass by: Gray, Sharena, Corrin (M), and Palla.](./examples/ike_to_roy.gif  "Ike to Roy")

![Interpolating between Mythic Loki and Loki. We pass by: Valentine's Leo, Winter Seidr, Summer Micaiah, Scion Julia, Knoll, Spring Palla, and New Year's Takumi.](./examples/mythic_loki_to_loki.gif "Mythic Loki to Loki")