;**********************************************************
; ROTACIO3.ASM
;
; Este programa ejecuta todas las rotaciones de un número
; de bits con una variable de tipo palabra doble. Las
; operaciones de rotación están implementadas mediante
; macros y se encuentran en el archivo ROTA_N.INC. Este

; programa ilustra el uso de la directiva include.
;**********************************************************

;****** CÓDIGO DE INICIO **********************************

        ideal
        dosseg
        model   small
        stack   256

;****** INCLUSIÓN DE ARCHIVOS *****************************

        include "rota_n.inc"

;****** VARIABLES DEL PROGRAMA ****************************

        dataseg
codsal  db      0
r_rcl   dd      ?
r_rcr   dd      ?
r_rol   dd      ?
r_ror   dd      ?

;****** CÓDIGO DEL PROGRAMA *******************************

        codeseg
inicio:
        mov     ax, @data               ; Inicializa el
        mov     ds, ax                  ; segmento de datos

; Rotación a la izquierda a través de la bandera de acarreo
        rcldwn  r_rcl, 2                ; rcl r_rcl, 2

; Rotación a la derecha a través de la bandera de acarreo
        rcrdwn  r_rcr, 2                ; rcr r_rcr, 2

; Rotación a la izquierda
        roldwn  r_rol, 2                ; rol r_rol, 2

; Rotación a la derecha
        rordwn  r_ror, 2                ; ror r_ror, 2

salir:
        mov     ah, 04Ch
        mov     al, [codsal]
        int     21h

;****** CÓDIGO DE TERMINACIÓN *****************************

        end     inicio
