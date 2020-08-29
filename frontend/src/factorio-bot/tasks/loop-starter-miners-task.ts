import {FactorioBot, MAX_ITEM_INVENTORY} from "@/factorio-bot/bot";
import {Store} from "vuex";
import {State} from "@/store";
import {createTask, executeTask, registerTaskRunner, Task, TaskStatus, updateTaskStatus} from "@/factorio-bot/task";
import {FactorioApi} from "@/factorio-bot/restApi";
import {Entities, InventoryType} from "@/factorio-bot/types";
import {createGatherTask} from "@/factorio-bot/tasks/gather-task";
import {
    fuelableRequestEntitiesFromWorld,
    fuelRequestEntitiesFromWorld,
    sleep,
    sortEntitiesByDistanceTo,
    targetRequestEntitiesFromWorld
} from "@/factorio-bot/util";

const TASK_TYPE = 'loop-starter-miners'

type TaskData = {
    fuelName: string,
    name: string,
    count: number,
}

async function executeThisTask(store: Store<State>, bots: FactorioBot[], task: Task): Promise<void> {
    const data: TaskData = task.data as TaskData
    if (bots.length === 0) {
        return
    }
    // bots.sort(sortBotsByInventory([data.name]))
    const bot = bots[0]
    let remaining = data.count - bot.mainInventory(data.name)
    while (remaining > 0) {
        store.commit('updateTask', updateTaskStatus(task, TaskStatus.STARTED));
        const targetInventories = await FactorioApi.inventoryContentsAt(targetRequestEntitiesFromWorld(store.state.world, data.name))
        targetInventories.sort(sortEntitiesByDistanceTo(bot.player().position))
        // console.log('targetInventories', targetInventories)
        // first get all target items
        for (const entity of targetInventories) {
            if (entity.outputInventory && entity.outputInventory[data.name]) {
                const take = entity.outputInventory[data.name]
                store.commit('updateTask', updateTaskStatus(task, TaskStatus.WALKING));
                try {
                    await bot.removeFromInventory(entity.name, entity.position,
                        data.name === Entities.coal || data.name === Entities.stone ?
                            InventoryType.chest_or_fuel : InventoryType.furnace_result, data.name, take)
                } catch (err) {
                    // ignore errors here ...
                }
                store.commit('updateTask', updateTaskStatus(task, TaskStatus.STARTED));
            }
        }
        remaining = data.count - bot.mainInventory(data.name)
        if (remaining <= 0) {
            break
        }
        // then use up all of our fuel
        const targetFuel = bot.mainInventory(data.fuelName) > 50 ? 10 : 5
        const minFuel = 2
        const fuelableInventories = await FactorioApi.inventoryContentsAt(fuelableRequestEntitiesFromWorld(store.state.world))
        fuelableInventories.sort(sortEntitiesByDistanceTo(bot.player().position))
        // console.log('fuelableInventories', fuelableInventories)
        for (const entity of fuelableInventories) {
            const entityCoal = (entity.fuelInventory || {})[data.fuelName] || 0
            if (entityCoal < minFuel) {
                const toInsert = Math.min(bot.mainInventory(data.fuelName), targetFuel - entityCoal)
                if (toInsert === 0) {
                    break
                }
                store.commit('updateTask', updateTaskStatus(task, TaskStatus.WALKING));
                try {
                    await bot.insertToInventory(entity.name, entity.position,
                        InventoryType.chest_or_fuel, data.fuelName, toInsert)
                } catch (err) {
                    // ignore errors here ...
                }
                store.commit('updateTask', updateTaskStatus(task, TaskStatus.STARTED));
            }
        }
        // then gather more fuel
        if (data.fuelName === Entities.coal && store.state.world.starterCoalLoops) {
            const coalLoopInventories = await FactorioApi.inventoryContentsAt(fuelRequestEntitiesFromWorld(store.state.world))
            coalLoopInventories.sort(sortEntitiesByDistanceTo(bot.player().position))
            // console.log('coalLoopInventories', coalLoopInventories)
            for(const coalMiner of coalLoopInventories) {
                const minerFuel = (coalMiner.fuelInventory || {})[data.fuelName] || 0
                const needs = MAX_ITEM_INVENTORY - bot.mainInventory(Entities.coal)
                const take = Math.min(needs, minerFuel-1)
                if (take > 0) {
                    store.commit('updateTask', updateTaskStatus(task, TaskStatus.WALKING));
                    try {
                        await bot.removeFromInventory(coalMiner.name, coalMiner.position, InventoryType.chest_or_fuel, Entities.coal, take)
                    } catch (err) {
                        // ignore errors here ...
                    }
                    store.commit('updateTask', updateTaskStatus(task, TaskStatus.STARTED));
                }
            }
        } else if(bot.mainInventory(data.fuelName) === 0) {
            const subtask = await createGatherTask(store, data.fuelName, 10)
            store.commit('addSubTask', {id: task.id, task: subtask})
            store.commit('updateTask', updateTaskStatus(task, TaskStatus.WAITING));
            await executeTask(store, bots, subtask)
        }
        remaining = data.count - bot.mainInventory(data.name)
        if (remaining <= 0) {
            break
        }
        store.commit('updateTask', updateTaskStatus(task, TaskStatus.SLEEPING));
        await sleep(2000)
    }
}

registerTaskRunner(TASK_TYPE, executeThisTask)

export async function createLoopStarterMinersTask(store: Store<State>, fuelName: string, name: string, count: number): Promise<Task> {
    const data: TaskData = {
        fuelName,
        name,
        count,
    }
    return createTask(TASK_TYPE, `Loop Starter Miners until ${name} x ${count}`, data)
}