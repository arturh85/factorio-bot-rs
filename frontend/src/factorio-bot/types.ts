export type PlaceEntity = {
 position: Position;
 direction: number;
}
export enum InventoryType {
 chest_or_fuel = 1,
 furnace_source = 2, // or lab input apparently
 furnace_result = 3,
}
export enum Direction {
 north,
 northeast,
 east,
 southeast,
 south,
 southwest,
 west,
 northwest,
}
export enum Entities {
 rockHuge = "rock-huge",
 water = "water",
 coal = "coal",
 stone = "stone",
 ironOre = "iron-ore",
 copperOre = "copper-ore",
 burnerMiningDrill = "burner-mining-drill",
 stoneFurnace = "stone-furnace",
 offshorePump = "offshore-pump",
 ironPlate = "iron-plate",
 copperPlate = "copper-plate",
 stoneBrick = "stone-brick",
 ironChest = "iron-chest",
 steamEngine = "steam-engine",
 boiler = "boiler",
 smallElectricPole = "small-electric-pole",
 pipe = "pipe",
 pipeToGround = "pipe-to-ground",
 transportBelt = "transport-belt",
 lab = "lab",
 automationSciencePack = "automation-science-pack",
 deadGreyTrunk = "dead-grey-trunk",
 wood = "wood",
}
export enum Technologies {
 automation = "automation",
 logistics = "logistics",
 logisticSciencePack = "logistic-science-pack",
 rocketSilo = "rocket-silo",
}
export type FactorioPlayerById = { [playerIdString: string]: FactorioPlayer };
export type FactorioRecipeByName = { [name: string]: FactorioRecipe };
export type FactorioTechnologyByName = { [name: string]: FactorioTechnology };
export type FactorioEntityPrototypeByName = { [name: string]: FactorioEntityPrototype };
export type FactorioItemPrototypeByName = { [name: string]: FactorioItemPrototype };
export type FactorioInventory = { [name: string]: number };

export type StarterMinerFurnace = {
 minerPosition: Position
 minerType: string
 furnacePosition: Position
 furnaceType: string
 oreName: string
 plateName: string
}
export type StarterMinerChest = {
 minerPosition: Position
 minerType: string
 chestPosition: Position
 chestType: string
 oreName: string
}
export type StarterCoalLoop = {
 minerPosition: Position
 minerType: string
}
export type World = {
 starterMinerFurnaces: StarterMinerFurnace[] | null
 starterMinerChests: StarterMinerChest[] | null
 starterCoalLoops: StarterCoalLoop[] | null
 starterOffshorePump: Position | null
 starterSteamEngineBlueprints: FactorioEntity[][] | null
 starterLabs: Position[] | null
}
export type FactorioBlueprintResult = {
 blueprint: FactorioBlueprint
}
export type FactorioBlueprintIcon = {
 index: number,
 signal: {
 name: string,
 type: string
 }
}
export type FactorioBlueprint = {
 entities: FactorioEntity[],
 icons: FactorioBlueprintIcon[],
 item: string,
 label: string,
 label_color: string,
 version: string
}
// --- AUTOGENERATED from types.rs starting here - do not remove this line ---
export type RequestEntity = { name: string; position: Position };
export type FactorioTile = { name: string; playerCollidable: boolean; position: Position };
export type FactorioTechnology = { name: string; enabled: boolean; upgrade: boolean; researched: boolean; prerequisites: string [] | null; researchUnitIngredients: FactorioIngredient []; researchUnitCount: number; researchUnitEnergy: number; order: string; level: number; valid: boolean };
export type FactorioForce = { name: string; forceId: number; currentResearch: string | null; researchProgress: number | null; technologies: { [key: string]: FactorioTechnology } };
export type InventoryResponse = { name: string; position: Position; outputInventory: { [key: string]: number } | null; fuelInventory: { [key: string]: number } | null };
export type FactorioRecipe = { name: string; valid: boolean; enabled: boolean; category: string; ingredients: FactorioIngredient []; products: FactorioProduct []; hidden: boolean; energy: number; order: string; group: string; subgroup: string };
export type FactorioIngredient = { name: string; ingredientType: string; amount: number };
export type FactorioProduct = { name: string; productType: string; amount: number; probability: number };
export type FactorioPlayer = { playerId: number; position: Position; mainInventory: { [key: string]: number } };
export type ChunkPosition = { x: number; y: number };
export type Position = { x: number; y: number };
export type Rect = { leftTop: Position; rightBottom: Position };
export type FactorioChunk = { objects: ChunkObject []; resources: ChunkResource []; tiles: FactorioTile [] };
export type ChunkObject = { name: string; position: Position; direction: string; boundingBox: Rect; outputInventory: { [key: string]: number } | null; fuelInventory: { [key: string]: number } | null };
export type ChunkResource = { name: string; position: Position };
export type FactorioGraphic = { entityName: string; imagePath: string; width: number; height: number };
export type FactorioEntity = { name: string; entityType: string; position: Position; amount: number | null; ghostName: string | null; ghostType: string | null };
export type FactorioEntityPrototype = { name: string; entityType: string; collisionMask: string [] | null; collisionBox: Rect; mineResult: { [key: string]: number } | null };
export type FactorioItemPrototype = { name: string; itemType: string; stackSize: number; fuelValue: number; placeResult: string; group: string; subgroup: string };
export type FactorioResult = { success: boolean; output: string [] };
