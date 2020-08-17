import type {
    FactorioBlueprint,
    FactorioEntity, FactorioEntityPrototypeByName, FactorioInventory,
    FactorioRecipeByName,
    FactorioTechnologyByName,
    FactorioTile,
    PlaceEntity,
    Position, Rect,
    World,
} from "@/factorio-bot/types";
import {Direction, Entities} from "@/factorio-bot/types";
import {FactorioBot} from "@/factorio-bot/bot";

export const emptyWorld: World = {
    starterMinerFurnaces: null,
    starterMinerChests: null,
    starterCoalLoops: null,
    starterOffshorePump: null,
    starterLabs: null,
    starterSteamEngineBlueprints: null,
};

export function countEntitiesFromBlueprint(blueprint: FactorioBlueprint): FactorioInventory {
    const inventory: FactorioInventory = {}
    for(const entity of blueprint.entities) {
        if (inventory[entity.name]) {
            inventory[entity.name] += 1
        } else {
            inventory[entity.name] = 1
        }
    }
    return inventory
}

export function movePositionInDirection(
    position: Position,
    direction: Direction,
    offset = 1
): Position {
    switch (direction) {
        case Direction.north:
            return {x: position.x, y: position.y - offset};
        case Direction.northeast:
            return {x: position.x - offset, y: position.y - offset};
        case Direction.northwest:
            return {x: position.x + offset, y: position.y - offset};
        case Direction.south:
            return {x: position.x, y: position.y + offset};
        case Direction.southeast:
            return {x: position.x - offset, y: position.y + offset};
        case Direction.southwest:
            return {x: position.x + offset, y: position.y + offset};
        case Direction.east:
            return {x: position.x - offset, y: position.y};
        case Direction.west:
            return {x: position.x + offset, y: position.y};
        default:
            throw new Error("impossible!");
    }
}

export function sortEntitiesByDistanceTo(position: Position): (a: FactorioEntity|FactorioTile, b: FactorioEntity|FactorioTile) => number {
    return (
        a: FactorioEntity | FactorioTile,
        b: FactorioEntity | FactorioTile
    ) => {
        const d1 = distance(position, a.position);
        const d2 = distance(position, b.position);
        return d1 - d2;
    };
}

export function floorPosition(position: Position): Position {
    return {x: Math.floor(position.x), y: Math.floor(position.y)}
}

export function positionEqual(a: Position, b: Position): boolean {
    return a.x === b.x && a.y === b.y
}

export function findFreeRect(entities: FactorioEntity[],
                             prototypes: FactorioEntityPrototypeByName,
                             oreName: string,
                             nearest: Position,
                             width: number,
                             height: number,
                             excludePositions: Position[] = [],
): Position | null {
    const oreEntities = entities.filter(entity => entity.name === oreName)
    oreEntities.sort(sortEntitiesByDistanceTo(nearest));
    const collisionRects = entities.filter(entity => entity.name !== oreName && prototypes[entity.name])
        .map(entity => entityRect(floorPosition(entity.position), prototypes[entity.name].collisionBox))
        .concat(excludePositions.map(position => entityRect(position, {leftTop: {x: -1, y: -1}, rightBottom: {x: 1, y: 1}})))
    for (const candidate of oreEntities) {
        const position = floorPosition(candidate.position)
        let valid = true
        for(let x = 0; x < width; x++) {
            for (let y = 0; y < height; y++) {
                const testPosition: Position = {x: position.x + x, y: position.y + y}
                const foundInPossiblePositions = (x == 0 && y == 0) || oreEntities.find(entity => positionEqual(testPosition, floorPosition(entity.position)))
                if (!foundInPossiblePositions) {
                    valid = false
                    break
                }
                const collidesWith = collisionRects.find(rect => posInRect(testPosition, rect))
                if (collidesWith) {
                    valid = false
                    break
                }
            }
            if(!valid) {
                break
            }
        }
        if (valid) {
            return position
        }
    }
    return null
}

export function entityRect(entityPosition: Position, entityCollisionBox: Rect): Rect {
    return {
        leftTop: {
            x: Math.floor(entityPosition.x + entityCollisionBox.leftTop.x),
            y: Math.floor(entityPosition.y + entityCollisionBox.leftTop.y),
        },
        rightBottom: {
            x: Math.ceil(entityPosition.x + entityCollisionBox.rightBottom.x),
            y: Math.ceil(entityPosition.y + entityCollisionBox.rightBottom.y),
        }
    }
}

export function posInRect(position: Position, rect: Rect): boolean {
    return position.x >= rect.leftTop.x &&
        position.x <= rect.rightBottom.x &&
        position.y >= rect.leftTop.y &&
        position.y <= rect.rightBottom.y
}

const reduceToMainInventoryCount = (bot: FactorioBot) => (cnt: number, itemName: string) => {
    return cnt + bot.mainInventory(itemName)
}


export function sortBotsByInventory(items: string[]): (a: FactorioBot, b: FactorioBot) => number {
    return (
        a: FactorioBot,
        b: FactorioBot
    ) => {
        const d1 = items.reduce(reduceToMainInventoryCount(a), 0)
        const d2 = items.reduce(reduceToMainInventoryCount(b), 0)
        return d1 - d2;
    };
}

export function nextResearch(
    technologies: FactorioTechnologyByName,
    target: string
): string {
    const prerequisites = (technologies[target].prerequisites || []).filter(
        (name) => !technologies[name].researched
    );
    if (prerequisites.length === 0) {
        return target;
    }
    return nextResearch(technologies, prerequisites[0]);
}

export function sleep(ms: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, ms));
}

export function positionLabel(position: Position): string {
    return `[ ${Math.round(position.x)}, ${Math.round(position.y)} ]`
}

export function distance(pos1: Position, pos2: Position): number {
    return Math.sqrt(Math.pow(pos1.x - pos2.x, 2) + Math.pow(pos1.y - pos2.y, 2));
}


export function placeEntitiesForCoalMinerLoop(
    topLeft: Position,
    count: number
): PlaceEntity[] {
    const coalMiners = [];
    const placePosition = {...topLeft};
    for (let y = 0; y < 2; y++) {
        placePosition.y = topLeft.y + y * 2;
        for (let x = 0; x < count / 2; x++) {
            let direction = y === 0 ? 2 : 6; // right/east in first row, left/west in second
            // if right top corner
            if (y === 0 && x === count / 2 - 1) {
                direction = 4; // down/south
            } else if (y == 1 && x === 0) {
                direction = 0; // up/north
            }
            placePosition.x = topLeft.x + x * 2;
            coalMiners.push({
                position: {...placePosition},
                direction: direction,
            });
        }
    }
    return coalMiners;
}

export function missingIngredients(
    recipes: FactorioRecipeByName,
    inventory: FactorioInventory,
    recipeName: string,
    count: number,
    includeSmeltingOres = false
): FactorioInventory {
    const missing: any = {};
    const recipe = recipes[recipeName];
    if (inventory[recipeName]) {
        const inventoryCount = Math.min(inventory[recipeName], count);
        count -= inventoryCount;
        inventory[recipeName] -= inventoryCount;
    }
    if (!recipe) {
        return count > 0 ? {[recipeName]: count} : {};
    }
    const addToMissing = (name: string, count: number) => {
        if (inventory[name]) {
            const inventoryCount = Math.min(inventory[name], count);
            count -= inventoryCount;
            inventory[name] -= inventoryCount;
        }
        if (missing[name]) {
            missing[name] += count;
        } else {
            missing[name] = count;
        }

        if (missing[name] === 0) {
            delete missing[name];
        }
    };
    for (const ingredient of recipe.ingredients) {
        const ingredientRecipe = recipes[ingredient.name];
        if (
            ingredientRecipe &&
            (ingredientRecipe.category === "crafting" || includeSmeltingOres)
        ) {
            const missingForIngredient = missingIngredients(
                recipes,
                inventory,
                ingredient.name,
                ingredient.amount * count,
                includeSmeltingOres
            );
            for (const missingForIngredientName of Object.keys(
                missingForIngredient
            )) {
                const missingForIngredientIngredient =
                    missingForIngredient[missingForIngredientName];
                addToMissing(missingForIngredientName, missingForIngredientIngredient);
            }
        } else {
            addToMissing(ingredient.name, ingredient.amount * count);
        }
    }
    return missing;
}

export function formatDuration(ms: number): string {
    const secondsTotal = Math.round(ms/1000)
    const minutes = Math.floor(secondsTotal / 60)
    const seconds = secondsTotal % 60
    return `${minutes}:${String(seconds).padStart(2, '0')}`
}