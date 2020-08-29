import {FactorioBot} from "@/factorio-bot/bot";
import {Store} from "vuex";
import {State} from "@/store";
import {createTask, registerTaskRunner, Task} from "@/factorio-bot/task";
import {Direction, Entities, InventoryType, Position, StarterMinerFurnace} from "@/factorio-bot/types";

const TASK_TYPE = 'place-starter-miner-furnace'

type TaskData = {
    minerName: string,
    furnaceName: string,
    minerPosition: Position,
    furnacePosition: Position,
    plateName: string,
    oreName: string,
}

async function executeThisTask(store: Store<State>, bots: FactorioBot[], task: Task): Promise<void> {
    const data: TaskData = task.data as TaskData
    const bot = bots[0]
    const minerEntity = await bot.placeEntity(data.minerName, data.minerPosition, Direction.south); // place down/south
    if (bot.mainInventory(Entities.coal) > 2) {
        await bot.insertToInventory(data.minerName, minerEntity.position, InventoryType.chest_or_fuel, Entities.coal, 2)
    }
    const furnaceEntity = await bot.placeEntity(data.furnaceName, data.furnacePosition, Direction.north); // place up/north but doesnt matter here
    if (bot.mainInventory(Entities.coal) > 2) {
        await bot.insertToInventory(data.furnaceName, furnaceEntity.position, InventoryType.chest_or_fuel, Entities.coal, 2)
    }

    const minerFurnace: StarterMinerFurnace = {
        minerPosition: minerEntity.position,
        furnacePosition: furnaceEntity.position,
        minerType: data.minerName,
        furnaceType: data.furnaceName,
        plateName: data.plateName,
        oreName: data.oreName
    }
    store.commit("addStarterMinerFurnace", minerFurnace)
}

registerTaskRunner(TASK_TYPE, executeThisTask)

export async function createPlaceStarterMinerFurnaceTask(store: Store<State>, minerName: string,
                                                         furnaceName: string,
                                                         minerPosition: Position,
                                                         furnacePosition: Position,plateName: string,
                                                         oreName: string,): Promise<Task> {
    const data: TaskData = {
        minerName,
        furnaceName,
        minerPosition,
        furnacePosition,
        plateName,
        oreName,
    }
    return createTask(TASK_TYPE, `Place Starter Miner Furnace`, data)
}