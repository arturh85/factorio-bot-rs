import type {
    FactorioBlueprintResult,
    FactorioEntity, FactorioEntityPrototypeByName,
    FactorioForce, FactorioItemPrototypeByName,
    FactorioPlayer,
    FactorioRecipeByName,
    FactorioTile,
    InventoryResponse,
    InventoryType,
    Position, Rect,
    RequestEntity,
} from "@/factorio-bot/types";
import {Direction} from "@/factorio-bot/types";
import {baseUrl} from "@/environment";
import {positionParam, rectParam} from "@/factorio-bot/util";

const fetch_retry = async (url: string, n: number, options: any = null): Promise<any> => {
    try {
        return await fetch(url, options)
    } catch(err) {
        if (n === 1) throw err;
        return await fetch_retry(url, n - 1, options);
    }
};

export class FactorioApi {
    static async insertToInventory(
        playerId: number,
        entityName: string,
        entityPosition: Position,
        inventoryType: InventoryType,
        itemName: string,
        itemCount: number
    ): Promise<FactorioPlayer> {
        const position = `${entityPosition.x},${entityPosition.y}`;
        const response = await fetch(
            `${baseUrl}/api/${playerId}/insertToInventory?entity_name=${entityName}&entity_position=${position}&inventory_type=${inventoryType}&item_name=${itemName}&item_count=${Math.floor(
                itemCount
            )}`
        );

        return await response.json();
    }

    static async removeFromInventory(
        playerId: number,
        entityName: string,
        entityPosition: Position,
        inventoryType: InventoryType,
        itemName: string,
        itemCount: number
    ): Promise<FactorioPlayer> {
        const position = `${entityPosition.x},${entityPosition.y}`;
        const response = await fetch(
            `${baseUrl}/api/${playerId}/removeFromInventory?entity_name=${entityName}&entity_position=${position}&inventory_type=${inventoryType}&item_name=${itemName}&item_count=${itemCount}`
        );
        return await response.json();
    }

    static async placeEntity(
        playerId: number,
        itemName: string,
        _placePosition: Position,
        placeDirection: number
    ): Promise<{ player: FactorioPlayer; entity: FactorioEntity }> {
        const position = `${_placePosition.x},${_placePosition.y}`;
        const response = await fetch(
            `${baseUrl}/api/${playerId}/placeEntity?item=${itemName}&position=${position}&direction=${placeDirection}`
        );
        return await response.json();
    }

    static async placeBlueprint(
        playerId: number,
        blueprint: string,
        _placePosition: Position,
        placeDirection = 0,
        forceBuild = false,
        onlyGhosts = false
    ): Promise<{ player: FactorioPlayer; entities: FactorioEntity[] }> {
        const position = `${_placePosition.x},${_placePosition.y}`;
        const response = await fetch(
            `${baseUrl}/api/${playerId}/placeBlueprint?blueprint=${encodeURIComponent(
                blueprint
            )}&position=${position}&direction=${placeDirection}&force_build=${forceBuild}&only_ghosts=${onlyGhosts}`
        );
        return await response.json();
    }

    static async reviveGhost(
        playerId: number,
        name: string,
        position: Position
    ): Promise<{ player: FactorioPlayer; entity: FactorioEntity }> {
        const response = await fetch(
            `${baseUrl}/api/${playerId}/reviveGhost?name=${name}&position=${positionParam(position)}`
        );
        return await response.json();
    }

    static async cheatBlueprint(
        playerId: number,
        blueprint: string,
        placePosition: Position,
        placeDirection: number = Direction.north,
        forceBuild = false
    ): Promise<{ player: FactorioPlayer; entities: FactorioEntity[] }> {
        const response = await fetch(
            `${baseUrl}/api/${playerId}/cheatBlueprint?blueprint=${encodeURIComponent(
                blueprint
            )}&position=${positionParam(placePosition)}&direction=${placeDirection}&force_build=${forceBuild}`
        );
        return await response.json();
    }

    static async parseBlueprint(blueprint: string): Promise<FactorioBlueprintResult> {
        const response = await fetch(
            `${baseUrl}/api/parseBlueprint?blueprint=${encodeURIComponent(blueprint)}`
        );
        return await response.json();
    }

    static async saveServer(): Promise<void> {
        const response = await fetch(`${baseUrl}/api/serverSave`);
        await response.json();
    }

    static async findEntities(
        centerPosition: Position,
        radius: number,
        name: string|null = null,
        entityType: string|null = null,
    ): Promise<FactorioEntity[]> {
        let url = `${baseUrl}/api/findEntities?position=${positionParam(centerPosition)}&radius=${radius}`;
        if (name) {
            url += `&name=${encodeURIComponent(name)}`
        }
        if (entityType) {
            url += `&entity_type=${encodeURIComponent(entityType)}`
        }
        const response = await fetch(url);
        return await response.json();
    }

    static async findEntitiesInArea(
        area: Rect,
        name: string|null = null,
        entityType: string|null = null,
    ): Promise<FactorioEntity[]> {
        let url = `${baseUrl}/api/findEntities?area=${rectParam(area)}`;
        if (name) {
            url += `&name=${encodeURIComponent(name)}`
        }
        if (entityType) {
            url += `&entity_type=${encodeURIComponent(entityType)}`
        }
        const response = await fetch(url);
        return await response.json();
    }

    static async findTilesInArea(
        area: Rect,
        name: string | null = null
    ): Promise<FactorioTile[]> {
        let url = `${baseUrl}/api/findTiles?area=${rectParam(area)}`
        if (name) {
            url += `&name=${name}`
        }
        const response = await fetch(url);
        return await response.json();
    }

    static async findTiles(
        centerPosition: Position,
        radius: number,
        name: string
    ): Promise<FactorioTile[]> {
        let url = `${baseUrl}/api/findTiles?position=${positionParam(centerPosition)}&radius=${radius}`
        if (name) {
            url += `&name=${name}`
        }
        const response = await fetch(url);
        return await response.json();
    }

    static async findEntitiesByType(
        centerPosition: Position,
        radius: number,
        entityType: string
    ): Promise<FactorioEntity[]> {
        const position = positionParam(centerPosition);
        const response = await fetch(
            `${baseUrl}/api/findEntities?position=${position}&radius=${radius}&entity_type=${entityType}`
        );
        return await response.json();
    }

    static async mine(
        playerId: number,
        _position: Position,
        name: string,
        count: number
    ): Promise<FactorioPlayer> {
        const position = positionParam(_position);
        const response = await fetch(
            `${baseUrl}/api/${playerId}/mine?name=${name}&position=${position}&count=${count}`
        );
        return await response.json();
    }

    static async move(
        playerId: number,
        _position: Position,
        radius: number
    ): Promise<FactorioPlayer> {
        const position = positionParam(_position);
        const response = await fetch(
            `${baseUrl}/api/${playerId}/move?goal=${position}&radius=${radius}`
        );
        return await response.json();
    }

    static async craft(
        playerId: number,
        recipeName: string,
        count: number
    ): Promise<FactorioPlayer> {
        const response = await fetch(
            `${baseUrl}/api/${playerId}/craft?recipe=${recipeName}&count=${count}`
        );
        return await response.json();
    }

    static async inventoryContentsAt(
        entities: RequestEntity[]
    ): Promise<InventoryResponse[]> {
        if (entities.length === 0) {
            throw new Error("required zero inventories?");
        }
        const query = entities
            .map(
                (entity) => `${entity.name}@${entity.position.x},${entity.position.y}`
            )
            .join(";");
        const response = await fetch(
            `${baseUrl}/api/inventoryContentsAt?query=${query}`
        );
        return await response.json();
    }

    static async allRecipes(): Promise<FactorioRecipeByName> {
        const response = await fetch_retry(`${baseUrl}/api/recipes`, 3);
        return await response.json();
    }

    static async allPlayers(): Promise<FactorioPlayer[]> {
        const response = await fetch_retry(`${baseUrl}/api/players`, 3);
        return await response.json();
    }

    static async playerForce(): Promise<FactorioForce> {
        const response = await fetch_retry(`${baseUrl}/api/playerForce`, 3);
        return await response.json();
    }

    static async addResearch(technologyName: string): Promise<void> {
        const response = await fetch(
            `${baseUrl}/api/addResearch?tech=${technologyName}`
        );
        return await response.json();
    }

    static async storeMapData(key: string, value: unknown): Promise<void> {
        const response = await fetch(`${baseUrl}/api/storeMapData?key=${key}`, {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify(value),
        });
        return await response.json();
    }

    static async retrieveMapData<T>(key: string): Promise<T> {
        const response = await fetch(`${baseUrl}/api/retrieveMapData?key=${key}`);
        return await response.json();
    }

    static async cheatItem(
        playerId: number,
        itemName: string,
        itemCount: number
    ): Promise<FactorioPlayer> {
        const response = await fetch(
            `${baseUrl}/api/${playerId}/cheatItem?name=${itemName}&count=${itemCount}`
        );
        return await response.json();
    }

    static async cheatTechnology(tech: string): Promise<void> {
        const response = await fetch(`${baseUrl}/api/cheatTechnology?tech=${tech}`);
        return await response.json();
    }

    static async cheatAllTechnologies(): Promise<void> {
        const response = await fetch(`${baseUrl}/api/cheatAllTechnologies`);
        return await response.json();
    }

    static async allEntityPrototypes(): Promise<FactorioEntityPrototypeByName> {
        const response = await fetch(`${baseUrl}/api/entityPrototypes`);
        return await response.json();
    }
    static async allItemPrototypes(): Promise<FactorioItemPrototypeByName> {
        const response = await fetch(`${baseUrl}/api/itemPrototypes`);
        return await response.json();
    }
}
