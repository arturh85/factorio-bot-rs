import type {FactorioRecipeByName} from "@/factorio-bot/types";
import {missingIngredients, placeEntitiesForCoalMinerLoop} from "@/factorio-bot/util";

const allRecipes: FactorioRecipeByName = {
    inserter: {
        name: "inserter",
        valid: true,
        enabled: true,
        category: "crafting",
        ingredients: [
            {
                name: "iron-plate",
                ingredientType: "item",
                amount: 1,
            },
            {
                name: "iron-gear-wheel",
                ingredientType: "item",
                amount: 1,
            },
            {
                name: "electronic-circuit",
                ingredientType: "item",
                amount: 1,
            },
        ],
        products: [
            {
                name: "inserter",
                productType: "item",
                amount: 1,
                probability: 1.0,
            },
        ],
        hidden: false,
        energy: 0.5,
        order: "b[inserter]",
        group: "logistics",
        subgroup: "inserter",
    },
    "wooden-chest": {
        name: "wooden-chest",
        valid: true,
        enabled: true,
        category: "crafting",
        ingredients: [
            {
                name: "wood",
                ingredientType: "item",
                amount: 2,
            },
        ],
        products: [
            {
                name: "wooden-chest",
                productType: "item",
                amount: 1,
                probability: 1.0,
            },
        ],
        hidden: false,
        energy: 0.5,
        order: "a[items]-a[wooden-chest]",
        group: "logistics",
        subgroup: "storage",
    },
    "iron-chest": {
        name: "iron-chest",
        valid: true,
        enabled: true,
        category: "crafting",
        ingredients: [
            {
                name: "iron-plate",
                ingredientType: "item",
                amount: 8,
            },
        ],
        products: [
            {
                name: "iron-chest",
                productType: "item",
                amount: 1,
                probability: 1.0,
            },
        ],
        hidden: false,
        energy: 0.5,
        order: "a[items]-b[iron-chest]",
        group: "logistics",
        subgroup: "storage",
    },
    "stone-furnace": {
        name: "stone-furnace",
        valid: true,
        enabled: true,
        category: "crafting",
        ingredients: [
            {
                name: "stone",
                ingredientType: "item",
                amount: 5,
            },
        ],
        products: [
            {
                name: "stone-furnace",
                productType: "item",
                amount: 1,
                probability: 1.0,
            },
        ],
        hidden: false,
        energy: 0.5,
        order: "a[stone-furnace]",
        group: "production",
        subgroup: "smelting-machine",
    },
    "iron-plate": {
        name: "iron-plate",
        valid: true,
        enabled: true,
        category: "smelting",
        ingredients: [
            {
                name: "iron-ore",
                ingredientType: "item",
                amount: 1,
            },
        ],
        products: [
            {
                name: "iron-plate",
                productType: "item",
                amount: 1,
                probability: 1.0,
            },
        ],
        hidden: false,
        energy: 3.2,
        order: "b[iron-plate]",
        group: "intermediate-products",
        subgroup: "raw-material",
    },
    "firearm-magazine": {
        name: "firearm-magazine",
        valid: true,
        enabled: true,
        category: "crafting",
        ingredients: [
            {
                name: "iron-plate",
                ingredientType: "item",
                amount: 4,
            },
        ],
        products: [
            {
                name: "firearm-magazine",
                productType: "item",
                amount: 1,
                probability: 1.0,
            },
        ],
        hidden: false,
        energy: 1.0,
        order: "a[basic-clips]-a[firearm-magazine]",
        group: "combat",
        subgroup: "ammo",
    },
    lab: {
        name: "lab",
        valid: true,
        enabled: true,
        category: "crafting",
        ingredients: [
            {
                name: "iron-gear-wheel",
                ingredientType: "item",
                amount: 10,
            },
            {
                name: "electronic-circuit",
                ingredientType: "item",
                amount: 10,
            },
            {
                name: "transport-belt",
                ingredientType: "item",
                amount: 4,
            },
        ],
        products: [
            {
                name: "lab",
                productType: "item",
                amount: 1,
                probability: 1.0,
            },
        ],
        hidden: false,
        energy: 2.0,
        order: "g[lab]",
        group: "production",
        subgroup: "production-machine",
    },
    "small-electric-pole": {
        name: "small-electric-pole",
        valid: true,
        enabled: true,
        category: "crafting",
        ingredients: [
            {
                name: "wood",
                ingredientType: "item",
                amount: 1,
            },
            {
                name: "copper-cable",
                ingredientType: "item",
                amount: 2,
            },
        ],
        products: [
            {
                name: "small-electric-pole",
                productType: "item",
                amount: 2,
                probability: 1.0,
            },
        ],
        hidden: false,
        energy: 0.5,
        order: "a[energy]-a[small-electric-pole]",
        group: "logistics",
        subgroup: "energy-pipe-distribution",
    },
    "copper-plate": {
        name: "copper-plate",
        valid: true,
        enabled: true,
        category: "smelting",
        ingredients: [
            {
                name: "copper-ore",
                ingredientType: "item",
                amount: 1,
            },
        ],
        products: [
            {
                name: "copper-plate",
                productType: "item",
                amount: 1,
                probability: 1.0,
            },
        ],
        hidden: false,
        energy: 3.2,
        order: "c[copper-plate]",
        group: "intermediate-products",
        subgroup: "raw-material",
    },
    "iron-stick": {
        name: "iron-stick",
        valid: true,
        enabled: true,
        category: "crafting",
        ingredients: [
            {
                name: "iron-plate",
                ingredientType: "item",
                amount: 1,
            },
        ],
        products: [
            {
                name: "iron-stick",
                productType: "item",
                amount: 2,
                probability: 1.0,
            },
        ],
        hidden: false,
        energy: 0.5,
        order: "b[iron-stick]",
        group: "intermediate-products",
        subgroup: "intermediate-product",
    },
    "burner-mining-drill": {
        name: "burner-mining-drill",
        valid: true,
        enabled: true,
        category: "crafting",
        ingredients: [
            {
                name: "iron-plate",
                ingredientType: "item",
                amount: 3,
            },
            {
                name: "iron-gear-wheel",
                ingredientType: "item",
                amount: 3,
            },
            {
                name: "stone-furnace",
                ingredientType: "item",
                amount: 1,
            },
        ],
        products: [
            {
                name: "burner-mining-drill",
                productType: "item",
                amount: 1,
                probability: 1.0,
            },
        ],
        hidden: false,
        energy: 2.0,
        order: "a[items]-a[burner-mining-drill]",
        group: "production",
        subgroup: "extraction-machine",
    },
    boiler: {
        name: "boiler",
        valid: true,
        enabled: true,
        category: "crafting",
        ingredients: [
            {
                name: "pipe",
                ingredientType: "item",
                amount: 4,
            },
            {
                name: "stone-furnace",
                ingredientType: "item",
                amount: 1,
            },
        ],
        products: [
            {
                name: "boiler",
                productType: "item",
                amount: 1,
                probability: 1.0,
            },
        ],
        hidden: false,
        energy: 0.5,
        order: "b[steam-power]-a[boiler]",
        group: "production",
        subgroup: "energy",
    },
    "pipe-to-ground": {
        name: "pipe-to-ground",
        valid: true,
        enabled: true,
        category: "crafting",
        ingredients: [
            {
                name: "iron-plate",
                ingredientType: "item",
                amount: 5,
            },
            {
                name: "pipe",
                ingredientType: "item",
                amount: 10,
            },
        ],
        products: [
            {
                name: "pipe-to-ground",
                productType: "item",
                amount: 2,
                probability: 1.0,
            },
        ],
        hidden: false,
        energy: 0.5,
        order: "a[pipe]-b[pipe-to-ground]",
        group: "logistics",
        subgroup: "energy-pipe-distribution",
    },
    pipe: {
        name: "pipe",
        valid: true,
        enabled: true,
        category: "crafting",
        ingredients: [
            {
                name: "iron-plate",
                ingredientType: "item",
                amount: 1,
            },
        ],
        products: [
            {
                name: "pipe",
                productType: "item",
                amount: 1,
                probability: 1.0,
            },
        ],
        hidden: false,
        energy: 0.5,
        order: "a[pipe]-a[pipe]",
        group: "logistics",
        subgroup: "energy-pipe-distribution",
    },
    radar: {
        name: "radar",
        valid: true,
        enabled: true,
        category: "crafting",
        ingredients: [
            {
                name: "iron-plate",
                ingredientType: "item",
                amount: 10,
            },
            {
                name: "iron-gear-wheel",
                ingredientType: "item",
                amount: 5,
            },
            {
                name: "electronic-circuit",
                ingredientType: "item",
                amount: 5,
            },
        ],
        products: [
            {
                name: "radar",
                productType: "item",
                amount: 1,
                probability: 1.0,
            },
        ],
        hidden: false,
        energy: 0.5,
        order: "d[radar]-a[radar]",
        group: "combat",
        subgroup: "defensive-structure",
    },
    "stone-brick": {
        name: "stone-brick",
        valid: true,
        enabled: true,
        category: "smelting",
        ingredients: [
            {
                name: "stone",
                ingredientType: "item",
                amount: 2,
            },
        ],
        products: [
            {
                name: "stone-brick",
                productType: "item",
                amount: 1,
                probability: 1.0,
            },
        ],
        hidden: false,
        energy: 3.2,
        order: "a[stone-brick]",
        group: "logistics",
        subgroup: "terrain",
    },
    "electric-mining-drill": {
        name: "electric-mining-drill",
        valid: true,
        enabled: true,
        category: "crafting",
        ingredients: [
            {
                name: "iron-plate",
                ingredientType: "item",
                amount: 10,
            },
            {
                name: "iron-gear-wheel",
                ingredientType: "item",
                amount: 5,
            },
            {
                name: "electronic-circuit",
                ingredientType: "item",
                amount: 3,
            },
        ],
        products: [
            {
                name: "electric-mining-drill",
                productType: "item",
                amount: 1,
                probability: 1.0,
            },
        ],
        hidden: false,
        energy: 2.0,
        order: "a[items]-b[electric-mining-drill]",
        group: "production",
        subgroup: "extraction-machine",
    },
    pistol: {
        name: "pistol",
        valid: true,
        enabled: true,
        category: "crafting",
        ingredients: [
            {
                name: "iron-plate",
                ingredientType: "item",
                amount: 5,
            },
            {
                name: "copper-plate",
                ingredientType: "item",
                amount: 5,
            },
        ],
        products: [
            {
                name: "pistol",
                productType: "item",
                amount: 1,
                probability: 1.0,
            },
        ],
        hidden: false,
        energy: 5.0,
        order: "a[basic-clips]-a[pistol]",
        group: "combat",
        subgroup: "gun",
    },
    "transport-belt": {
        name: "transport-belt",
        valid: true,
        enabled: true,
        category: "crafting",
        ingredients: [
            {
                name: "iron-plate",
                ingredientType: "item",
                amount: 1,
            },
            {
                name: "iron-gear-wheel",
                ingredientType: "item",
                amount: 1,
            },
        ],
        products: [
            {
                name: "transport-belt",
                productType: "item",
                amount: 2,
                probability: 1.0,
            },
        ],
        hidden: false,
        energy: 0.5,
        order: "a[transport-belt]-a[transport-belt]",
        group: "logistics",
        subgroup: "belt",
    },
    "automation-science-pack": {
        name: "automation-science-pack",
        valid: true,
        enabled: true,
        category: "crafting",
        ingredients: [
            {
                name: "copper-plate",
                ingredientType: "item",
                amount: 1,
            },
            {
                name: "iron-gear-wheel",
                ingredientType: "item",
                amount: 1,
            },
        ],
        products: [
            {
                name: "automation-science-pack",
                productType: "item",
                amount: 1,
                probability: 1.0,
            },
        ],
        hidden: false,
        energy: 5.0,
        order: "a[automation-science-pack]",
        group: "intermediate-products",
        subgroup: "science-pack",
    },
    "offshore-pump": {
        name: "offshore-pump",
        valid: true,
        enabled: true,
        category: "crafting",
        ingredients: [
            {
                name: "iron-gear-wheel",
                ingredientType: "item",
                amount: 1,
            },
            {
                name: "electronic-circuit",
                ingredientType: "item",
                amount: 2,
            },
            {
                name: "pipe",
                ingredientType: "item",
                amount: 1,
            },
        ],
        products: [
            {
                name: "offshore-pump",
                productType: "item",
                amount: 1,
                probability: 1.0,
            },
        ],
        hidden: false,
        energy: 0.5,
        order: "b[fluids]-a[offshore-pump]",
        group: "production",
        subgroup: "extraction-machine",
    },
    "steam-engine": {
        name: "steam-engine",
        valid: true,
        enabled: true,
        category: "crafting",
        ingredients: [
            {
                name: "iron-plate",
                ingredientType: "item",
                amount: 10,
            },
            {
                name: "iron-gear-wheel",
                ingredientType: "item",
                amount: 8,
            },
            {
                name: "pipe",
                ingredientType: "item",
                amount: 5,
            },
        ],
        products: [
            {
                name: "steam-engine",
                productType: "item",
                amount: 1,
                probability: 1.0,
            },
        ],
        hidden: false,
        energy: 0.5,
        order: "b[steam-power]-b[steam-engine]",
        group: "production",
        subgroup: "energy",
    },
    "copper-cable": {
        name: "copper-cable",
        valid: true,
        enabled: true,
        category: "crafting",
        ingredients: [
            {
                name: "copper-plate",
                ingredientType: "item",
                amount: 1,
            },
        ],
        products: [
            {
                name: "copper-cable",
                productType: "item",
                amount: 2,
                probability: 1.0,
            },
        ],
        hidden: false,
        energy: 0.5,
        order: "a[copper-cable]",
        group: "intermediate-products",
        subgroup: "intermediate-product",
    },
    "repair-pack": {
        name: "repair-pack",
        valid: true,
        enabled: true,
        category: "crafting",
        ingredients: [
            {
                name: "iron-gear-wheel",
                ingredientType: "item",
                amount: 2,
            },
            {
                name: "electronic-circuit",
                ingredientType: "item",
                amount: 2,
            },
        ],
        products: [
            {
                name: "repair-pack",
                productType: "item",
                amount: 1,
                probability: 1.0,
            },
        ],
        hidden: false,
        energy: 0.5,
        order: "b[repair]-a[repair-pack]",
        group: "production",
        subgroup: "tool",
    },
    "electronic-circuit": {
        name: "electronic-circuit",
        valid: true,
        enabled: true,
        category: "crafting",
        ingredients: [
            {
                name: "iron-plate",
                ingredientType: "item",
                amount: 1,
            },
            {
                name: "copper-cable",
                ingredientType: "item",
                amount: 3,
            },
        ],
        products: [
            {
                name: "electronic-circuit",
                productType: "item",
                amount: 1,
                probability: 1.0,
            },
        ],
        hidden: false,
        energy: 0.5,
        order: "e[electronic-circuit]",
        group: "intermediate-products",
        subgroup: "intermediate-product",
    },
    "light-armor": {
        name: "light-armor",
        valid: true,
        enabled: true,
        category: "crafting",
        ingredients: [
            {
                name: "iron-plate",
                ingredientType: "item",
                amount: 40,
            },
        ],
        products: [
            {
                name: "light-armor",
                productType: "item",
                amount: 1,
                probability: 1.0,
            },
        ],
        hidden: false,
        energy: 3.0,
        order: "a[light-armor]",
        group: "combat",
        subgroup: "armor",
    },
    "iron-gear-wheel": {
        name: "iron-gear-wheel",
        valid: true,
        enabled: true,
        category: "crafting",
        ingredients: [
            {
                name: "iron-plate",
                ingredientType: "item",
                amount: 2,
            },
        ],
        products: [
            {
                name: "iron-gear-wheel",
                productType: "item",
                amount: 1,
                probability: 1.0,
            },
        ],
        hidden: false,
        energy: 0.5,
        order: "c[iron-gear-wheel]",
        group: "intermediate-products",
        subgroup: "intermediate-product",
    },
    "burner-inserter": {
        name: "burner-inserter",
        valid: true,
        enabled: true,
        category: "crafting",
        ingredients: [
            {
                name: "iron-plate",
                ingredientType: "item",
                amount: 1,
            },
            {
                name: "iron-gear-wheel",
                ingredientType: "item",
                amount: 1,
            },
        ],
        products: [
            {
                name: "burner-inserter",
                productType: "item",
                amount: 1,
                probability: 1.0,
            },
        ],
        hidden: false,
        energy: 0.5,
        order: "a[burner-inserter]",
        group: "logistics",
        subgroup: "inserter",
    },
};

describe("util", () => {
    describe("buildCoalMinerLoop(recipes, inventory, recipeName, count)", () => {
        it("should work for 2", () => {
            expect(placeEntitiesForCoalMinerLoop({x: 0, y: 0}, 2)).toEqual([
                {
                    position: {x: 0, y: 0},
                    direction: 4, // DOWN
                },
                {
                    position: {x: 0, y: 2},
                    direction: 0, // UP
                },
            ]);
        });
        it("should work for 4", () => {
            expect(placeEntitiesForCoalMinerLoop({x: 0, y: 0}, 4)).toEqual([
                {
                    position: {x: 0, y: 0},
                    direction: 2, // RIGHT
                },
                {
                    position: {x: 2, y: 0},
                    direction: 4, // DOWN
                },
                {
                    position: {x: 0, y: 2},
                    direction: 0, // UP
                },
                {
                    position: {x: 2, y: 2},
                    direction: 6, // LEFT
                },
            ]);
        });
    });
    describe("missingIngredients(recipes, inventory, recipeName, count)", () => {
        it("should work in basic case with no inventory and recursion", () => {
            expect(missingIngredients(allRecipes, {}, "wooden-chest", 1)).toEqual({
                wood: 2,
            });
        });
        it("should work in basic case with no inventory and recursion with multiply count", () => {
            expect(missingIngredients(allRecipes, {}, "wooden-chest", 4)).toEqual({
                wood: 8,
            });
        });
        it("should work in basic case with recursion", () => {
            expect(
                missingIngredients(allRecipes, {}, "burner-mining-drill", 1)
            ).toEqual({
                "iron-plate": 9,
                stone: 5,
            });
        });
        it("should with smelting", () => {
            expect(
                missingIngredients(allRecipes, {}, "burner-mining-drill", 1, true)
            ).toEqual({
                "iron-ore": 9,
                stone: 5,
            });
        });
        it("should multiply with count", () => {
            expect(
                missingIngredients(allRecipes, {}, "burner-mining-drill", 4)
            ).toEqual({
                "iron-plate": 36,
                stone: 20,
            });
        });
        it("should use inventory", () => {
            expect(
                missingIngredients(
                    allRecipes,
                    {"burner-mining-drill": 1, "iron-plate": 20},
                    "burner-mining-drill",
                    4
                )
            ).toEqual({
                "iron-plate": 7,
                stone: 15,
            });
        });
    });
});
