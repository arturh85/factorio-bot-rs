import {FactorioBot} from "@/factorio-bot/bot";
import {Store} from "vuex";
import {State} from "@/store";
import {createTask, executeTask, Task, taskRunnerByType, TaskStatus, updateTaskStatus} from "@/factorio-bot/task";
import {FactorioApi} from "@/factorio-bot/restApi";
import {Entities, InventoryType, RequestEntity} from "@/factorio-bot/types";
import {sleep, sortBotsByInventory} from "@/factorio-bot/util";
import {createGatherTask} from "@/factorio-bot/tasks/gather-task";

const TASK_TYPE = 'loop-starter-miner-furnace'

type TaskData = {
    fuelName: string,
    plateName: string,
    plateCount: number,
}

async function executeThisTask(store: Store<State>, bots: FactorioBot[], task: Task): Promise<void> {
    const data: TaskData = task.data as TaskData
    if (bots.length === 0) {
        return
    }
    // bots.sort(sortBotsByInventory([data.name]))
    const bot = bots[0]
    let remaining = data.plateCount
    while (remaining > 0) {
        store.commit('updateTask', updateTaskStatus(task, TaskStatus.STARTED));
        const inventoryContents = await FactorioApi.inventoryContentsAt((store.state.world.starterMinerFurnaces || [])
            .filter(minerFurnace => minerFurnace.plateName === data.plateName)
            .flatMap(minerFurnace => {
            const entities: RequestEntity[] = [];
            entities.push({
                name: minerFurnace.furnaceType,
                position: minerFurnace.furnacePosition,
            })
            entities.push({
                name: minerFurnace.minerType,
                position: minerFurnace.minerPosition,
            })
            return entities;
        }))
        // first get all plates
        for (const entity of inventoryContents) {
            if (entity.outputInventory && entity.outputInventory[data.plateName]) {
                const take = Math.min(remaining, entity.outputInventory[data.plateName])
                console.log('REMOVE ', entity.outputInventory[data.plateName], entity, take)
                await bot.removeFromInventory(entity.name, entity.position, InventoryType.furnace_result, data.plateName, take)
                remaining -= take
                if (remaining === 0) {
                    break
                }
            }
        }
        if (remaining === 0) {
            break
        }
        // then get enough fuel
        const targetFuel = bot.mainInventory(data.fuelName) > 40 ? 10 : 4
        const lowFuelEntities = inventoryContents.filter(entity => entity.fuelInventory && (!entity.fuelInventory[data.fuelName] || entity.fuelInventory[data.fuelName] < targetFuel))
        const lowFuelEntitiesFuel = lowFuelEntities.reduce((coal, entity) => {
            return coal + ((entity.fuelInventory || {})[data.fuelName] || 0)
        }, 0)
        const fuelNeeded = inventoryContents.length * targetFuel - lowFuelEntitiesFuel - bot.mainInventory(data.fuelName)

        if (fuelNeeded > 0) {
            const subtask = await createGatherTask(store, data.fuelName, fuelNeeded)
            store.commit('addSubTask', {id: task.id, task: subtask})
            store.commit('updateTask', updateTaskStatus(task, TaskStatus.WAITING));
            try {
                await executeTask(store, bots, subtask)
            } catch (err) {
                console.warn('subtask failed', err)
                store.commit('updateTask', updateTaskStatus(task, TaskStatus.FAILED));
                return
            }
        }

        // then insert fuel
        for (const entity of inventoryContents) {
            const entityCoal = (entity.fuelInventory || {})[data.fuelName] || 0
            if (entityCoal < targetFuel) {
                store.commit('updateTask', updateTaskStatus(task, TaskStatus.STARTED));
                try {
                    await bot.insertToInventory(entity.name, entity.position,
                        InventoryType.chest_or_fuel, data.fuelName, targetFuel - entityCoal)
                } catch (err) {
                    // ignore errors here ...
                }
            }
        }
        store.commit('updateTask', updateTaskStatus(task, TaskStatus.SLEEPING));
        await sleep(5000)
    }
}

taskRunnerByType[TASK_TYPE] = executeThisTask

export async function createLoopStarterMinerFurnaceTask(store: Store<State>, fuelName: string, plateName: string, plateCount: number): Promise<Task> {
    const data: TaskData = {
        fuelName,
        plateName,
        plateCount,
    }
    return createTask(TASK_TYPE, `Refill Starter Miner/Furnace until ${plateName} x ${plateCount}`, data)
}