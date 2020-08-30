import {FactorioBot} from "@/factorio-bot/bot";
import {Store} from "vuex";
import {State} from "@/store";
import {createTask, registerTaskRunner, Task} from "@/factorio-bot/task";
import {countEntitiesFromBlueprint, positionLabel} from "@/factorio-bot/util";
import {Direction, FactorioEntity, Position} from "@/factorio-bot/types";
import {FactorioApi} from "@/factorio-bot/restApi";
import {buildBotQueueToCraft, processBotQueue} from "@/factorio-bot/bot-queue";

const TASK_TYPE = 'build-blueprint'

type TaskData = {
    blueprintLabel: string,
    blueprintString: string,
    position: Position,
    direction: Direction,
    immediate: boolean,
}

async function executeThisTask(store: Store<State>, bots: FactorioBot[], task: Task): Promise<FactorioEntity[]> {
    const data: TaskData = task.data as TaskData
    const firstBot = bots[0]
    if (!data.immediate) {
        const result = await FactorioApi.parseBlueprint(data.blueprintString)
        const entities = countEntitiesFromBlueprint(result.blueprint)
        // each bot should first craft what it needs
        const queue = await buildBotQueueToCraft(store, task, bots, entities)
        await processBotQueue(store, queue, bots)
    }
    return await firstBot.placeBlueprint(
        data.blueprintString,
        data.position,
        data.direction,
        false,
        bots.slice(1).map(bot => bot.playerId)
    );
}

registerTaskRunner(TASK_TYPE, executeThisTask)

export async function createBuildBlueprint(store: Store<State>, blueprintLabel: string, blueprintString: string,
                                           position: Position,
                                           direction: Direction,
                                           immediate: boolean): Promise<Task> {
    const data: TaskData = {
        blueprintLabel,
        blueprintString,
        position,
        direction,
        immediate,
    }
    return createTask(TASK_TYPE, `Build Blueprint Lab '${blueprintLabel}' @ ${positionLabel(position)} (${direction})`, data)
}